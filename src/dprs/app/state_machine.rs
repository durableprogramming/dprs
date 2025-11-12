// The state_machine module manages the core application state for the
// Docker container management TUI. It defines the Container struct to store
// container information, AppEvent enum for user interactions, and AppState
// for maintaining the current application state including container list
// and selection. The implementation includes methods to navigate container
// lists, select containers, and refresh container data by querying the
// Docker CLI. This serves as the central data model for the application.

use crate::dprs::modes::{CommandState, Mode, SearchState, VisualSelection};
use ratatui::widgets::{ListState, TableState};
use std::collections::HashSet;
use std::io::Error;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Clone)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub status: String,
    pub ip_address: String,
    pub ports: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContainerFilter {
    Running,
    Recent,
    All,
}

impl ContainerFilter {
    pub fn display_name(&self) -> &'static str {
        match self {
            ContainerFilter::Running => "RUNNING",
            ContainerFilter::Recent => "RECENT",
            ContainerFilter::All => "ALL",
        }
    }
}

#[derive(Clone)]
pub struct ProgressModal {
    pub message: String,
    pub percentage: f32,
    pub active: bool,
}

pub enum AppEvent {
    SelectNext,
    SelectPrevious,
    Refresh,
    CopyIp,
    OpenBrowser,
    StopContainer,
    RestartContainer,
    Quit,
    EnterFilterMode,
    ExitFilterMode,
    ClearFilter,

    // Modal Events
    EnterNormalMode,
    EnterVisualMode,
    EnterCommandMode,
    EnterSearchMode,

    // Vim-style Navigation
    GoToFirst,
    GoToLast,
    WordNext,
    WordPrevious,
    HalfPageUp,
    HalfPageDown,
    NextSearchResult,
    PreviousSearchResult,

    // Visual Mode
    ExtendSelection,
    StopSelectedContainers,
    RestartSelectedContainers,

    // Additional actions
    ToggleTabular,

    // Command Mode
    ExecuteCommand,
    CancelCommand,
}

pub struct AppState {
    pub containers: Vec<Container>,
    pub list_state: ListState,
    pub table_state: TableState,
    pub tabular_mode: bool,
    pub compose_view_mode: bool,
    pub container_filter: ContainerFilter,
    pub filter_mode: bool,
    pub filter_text: String,
    pub filtered_containers: Vec<usize>,

    // Modal state
    pub mode: Mode,
    pub visual_selection: Option<VisualSelection>,
    pub command_state: CommandState,
    pub search_state: SearchState,
    pub last_normal_position: usize,

    // Progress modal
    pub progress_modal: ProgressModal,
    pub progress_receiver: Option<Receiver<ProgressUpdate>>,

    // Effects state
    pub previous_container_names: std::collections::HashSet<String>,
    pub new_container_indices: Vec<usize>,

    // Exit flag
    pub exit_requested: bool,
}

#[derive(Clone)]
pub enum ProgressUpdate {
    Update { message: String, percentage: f32 },
    Complete,
    Error(String),
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Formats IP addresses for display: comma-separated, max 3 IPs
fn format_ip_addresses(ip_string: &str) -> String {
    let ips: Vec<&str> = ip_string
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .collect();

    if ips.is_empty() {
        return String::new();
    }

    if ips.len() <= 3 {
        ips.join(", ")
    } else {
        format!("{}, ... (+{})", ips[..3].join(", "), ips.len() - 3)
    }
}

impl AppState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Self {
            containers: Vec::new(),
            list_state,
            table_state,
            tabular_mode: false,
            compose_view_mode: false,
            container_filter: ContainerFilter::Running,
            filter_mode: false,
            filter_text: String::new(),
            filtered_containers: Vec::new(),
            mode: Mode::Normal,
            visual_selection: None,
            command_state: CommandState::new(),
            search_state: SearchState::new(),
            last_normal_position: 0,
            progress_modal: ProgressModal {
                message: String::new(),
                percentage: 0.0,
                active: false,
            },
            progress_receiver: None,
            previous_container_names: HashSet::new(),
            new_container_indices: Vec::new(),
            exit_requested: false,
        }
    }

    pub fn next(&mut self) {
        let container_count = self.get_displayed_container_count();
        if container_count == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= container_count.saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let container_count = self.get_displayed_container_count();
        if container_count == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    container_count.saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.table_state.select(Some(i));
    }

    pub fn get_selected_container(&self) -> Option<&Container> {
        match self.list_state.selected() {
            Some(i) => {
                if self.filter_text.is_empty() {
                    self.containers.get(i)
                } else {
                    self.filtered_containers
                        .get(i)
                        .and_then(|&real_index| self.containers.get(real_index))
                }
            }
            None => None,
        }
    }

    pub fn refresh_containers(&mut self) -> Result<(), Error> {
        // Store previous container names for detecting new ones
        let previous_names: std::collections::HashSet<String> =
            self.containers.iter().map(|c| c.name.clone()).collect();

        self.containers.clear();
        self.new_container_indices.clear();

        // Build docker ps command based on container filter
        let mut args = vec!["ps"];

        match self.container_filter {
            ContainerFilter::Running => {
                // Default: only running containers
            }
            ContainerFilter::Recent => {
                // Show recently exited containers (last 10)
                args.push("-a");
                args.push("--filter");
                args.push("status=exited");
                args.push("--last");
                args.push("10");
            }
            ContainerFilter::All => {
                // Show all containers (running, stopped, exited)
                args.push("-a");
            }
        }

        args.push("--format");
        args.push("{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}");

        let output = Command::new("docker")
            .args(&args)
            .output()
            .map_err(|e| Error::other(format!("Failed to execute docker command: {}", e)))?;

        if !output.status.success() {
            return Err(Error::other(format!(
                "Docker command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let image = parts[1].to_string();
                let status = parts[2].to_string();
                let ports = parts[3].to_string();

                // Get container IP address
                let ip_output = Command::new("docker")
                    .arg("inspect")
                    .arg("--format")
                    .arg("{{range .NetworkSettings.Networks}}{{.IPAddress}} {{end}}")
                    .arg(name.as_str())
                    .output()
                    .map_err(|e| Error::other(format!("Failed to get container IP: {}", e)))?;

                let ip_address_raw = String::from_utf8_lossy(&ip_output.stdout)
                    .trim()
                    .to_string();

                // Format IPs: comma-separated, max 3 IPs
                let ip_address = format_ip_addresses(&ip_address_raw);

                let is_new = !previous_names.contains(&name);
                self.containers.push(Container {
                    name,
                    image,
                    status,
                    ip_address,
                    ports,
                });
                if is_new {
                    self.new_container_indices.push(self.containers.len() - 1);
                }
            }
        }

        // Update previous names for next refresh
        self.previous_container_names = self.containers.iter().map(|c| c.name.clone()).collect();

        // Reset selection if the list is empty or the current selection is invalid
        if self.containers.is_empty() {
            self.list_state.select(None);
            self.table_state.select(None);
        } else if self.list_state.selected().is_none()
            || self.list_state.selected().unwrap() >= self.containers.len()
        {
            self.list_state.select(Some(0));
            self.table_state.select(Some(0));
        }

        Ok(())
    }

    pub fn load_containers(&mut self) {
        let _ = self.refresh_containers();
    }

    pub fn enter_filter_mode(&mut self) {
        self.filter_mode = true;
    }

    pub fn exit_filter_mode(&mut self) {
        self.filter_mode = false;
    }

    pub fn update_filter(&mut self, text: String) {
        self.filter_text = text;
        self.apply_filter();
    }

    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.filtered_containers.clear();
        if !self.containers.is_empty() {
            self.list_state.select(Some(0));
            self.table_state.select(Some(0));
        }
    }

    fn apply_filter(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_containers.clear();
            return;
        }

        let filter_lower = self.filter_text.to_lowercase();
        self.filtered_containers = self
            .containers
            .iter()
            .enumerate()
            .filter(|(_, container)| {
                container.name.to_lowercase().contains(&filter_lower)
                    || container.image.to_lowercase().contains(&filter_lower)
                    || container.status.to_lowercase().contains(&filter_lower)
            })
            .map(|(i, _)| i)
            .collect();

        // Reset selection to first filtered item
        if !self.filtered_containers.is_empty() {
            self.list_state.select(Some(0));
            self.table_state.select(Some(0));
        } else {
            self.list_state.select(None);
            self.table_state.select(None);
        }
    }

    pub fn get_displayed_containers(&self) -> Vec<Container> {
        if self.filter_text.is_empty() {
            self.containers.clone()
        } else {
            self.filtered_containers
                .iter()
                .filter_map(|&i| self.containers.get(i))
                .cloned()
                .collect()
        }
    }

    pub fn get_displayed_container_count(&self) -> usize {
        if self.compose_view_mode {
            // In compose view mode, count projects instead of containers
            use crate::dprs::display::compose_view::group_containers_by_project;
            group_containers_by_project(self).len()
        } else if self.filter_text.is_empty() {
            self.containers.len()
        } else {
            self.filtered_containers.len()
        }
    }

    // Modal state management
    pub fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
        self.visual_selection = None;
        self.command_state.clear();
        self.search_state.clear();
        if let Some(selected) = self.list_state.selected() {
            self.last_normal_position = selected;
        }
    }

    pub fn enter_visual_mode(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            self.mode = Mode::Visual;
            self.visual_selection = Some(VisualSelection::new(selected));
        }
    }

    pub fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
        self.command_state.clear();
    }

    pub fn enter_search_mode(&mut self, forward: bool) {
        self.mode = Mode::Search;
        self.search_state.set_query(String::new(), forward);
    }

    pub fn is_in_visual_mode(&self) -> bool {
        matches!(self.mode, Mode::Visual)
    }

    pub fn is_in_command_mode(&self) -> bool {
        matches!(self.mode, Mode::Command)
    }

    pub fn is_in_search_mode(&self) -> bool {
        matches!(self.mode, Mode::Search)
    }

    // Vim-style navigation
    pub fn go_to_first(&mut self) {
        let container_count = self.get_displayed_container_count();
        if container_count > 0 {
            self.list_state.select(Some(0));
            self.table_state.select(Some(0));
            if let Some(ref mut selection) = self.visual_selection {
                selection.extend_to(0);
            }
        }
    }

    pub fn go_to_last(&mut self) {
        let container_count = self.get_displayed_container_count();
        if container_count > 0 {
            let last_index = container_count - 1;
            self.list_state.select(Some(last_index));
            self.table_state.select(Some(last_index));
            if let Some(ref mut selection) = self.visual_selection {
                selection.extend_to(last_index);
            }
        }
    }

    pub fn half_page_up(&mut self) {
        let container_count = self.get_displayed_container_count();
        if container_count == 0 {
            return;
        }

        if let Some(current) = self.list_state.selected() {
            let new_pos = current.saturating_sub(container_count / 2).max(0);
            self.list_state.select(Some(new_pos));
            self.table_state.select(Some(new_pos));
            if let Some(ref mut selection) = self.visual_selection {
                selection.extend_to(new_pos);
            }
        }
    }

    pub fn half_page_down(&mut self) {
        let container_count = self.get_displayed_container_count();
        if container_count == 0 {
            return;
        }

        if let Some(current) = self.list_state.selected() {
            let new_pos = (current + container_count / 2).min(container_count - 1);
            self.list_state.select(Some(new_pos));
            self.table_state.select(Some(new_pos));
            if let Some(ref mut selection) = self.visual_selection {
                selection.extend_to(new_pos);
            }
        }
    }

    pub fn word_next(&mut self) {
        // Move to next container with different name prefix
        let container_count = self.get_displayed_container_count();
        if container_count == 0 {
            return;
        }

        if let Some(current) = self.list_state.selected() {
            let containers = self.get_displayed_containers();
            if current >= containers.len() {
                return;
            }

            let current_name = &containers[current].name;
            let current_prefix = current_name.split('-').next().unwrap_or(current_name);

            for (i, container) in containers.iter().enumerate().skip(current + 1) {
                let name = &container.name;
                let prefix = name.split('-').next().unwrap_or(name);
                if prefix != current_prefix {
                    self.list_state.select(Some(i));
                    self.table_state.select(Some(i));
                    if let Some(ref mut selection) = self.visual_selection {
                        selection.extend_to(i);
                    }
                    return;
                }
            }

            // If no different prefix found, go to end
            self.go_to_last();
        }
    }

    pub fn word_previous(&mut self) {
        // Move to previous container with different name prefix
        let container_count = self.get_displayed_container_count();
        if container_count == 0 {
            return;
        }

        if let Some(current) = self.list_state.selected() {
            let containers = self.get_displayed_containers();
            if current >= containers.len() {
                return;
            }

            let current_name = &containers[current].name;
            let current_prefix = current_name.split('-').next().unwrap_or(current_name);

            for i in (0..current).rev() {
                let name = &containers[i].name;
                let prefix = name.split('-').next().unwrap_or(name);
                if prefix != current_prefix {
                    self.list_state.select(Some(i));
                    self.table_state.select(Some(i));
                    if let Some(ref mut selection) = self.visual_selection {
                        selection.extend_to(i);
                    }
                    return;
                }
            }

            // If no different prefix found, go to beginning
            self.go_to_first();
        }
    }

    pub fn get_selected_indices(&self) -> Vec<usize> {
        match &self.visual_selection {
            Some(selection) => {
                let mut indices: Vec<usize> = selection.selected_indices.iter().copied().collect();
                indices.sort();
                indices
            }
            None => {
                if let Some(selected) = self.list_state.selected() {
                    vec![selected]
                } else {
                    Vec::new()
                }
            }
        }
    }

    pub fn perform_search(&mut self, query: &str) {
        use regex::Regex;

        self.search_state.matches.clear();

        if query.is_empty() {
            return;
        }

        let containers = self.get_displayed_containers();

        // Try regex first, fall back to simple string search
        let matches = if let Ok(re) = Regex::new(query) {
            containers
                .iter()
                .enumerate()
                .filter(|(_, container)| {
                    re.is_match(&container.name)
                        || re.is_match(&container.image)
                        || re.is_match(&container.status)
                })
                .map(|(i, _)| i)
                .collect()
        } else {
            let query_lower = query.to_lowercase();
            containers
                .iter()
                .enumerate()
                .filter(|(_, container)| {
                    container.name.to_lowercase().contains(&query_lower)
                        || container.image.to_lowercase().contains(&query_lower)
                        || container.status.to_lowercase().contains(&query_lower)
                })
                .map(|(i, _)| i)
                .collect()
        };

        self.search_state.update_matches(matches);
        self.search_state.last_query = query.to_string();
    }

    pub fn next_search_result(&mut self) {
        if let Some(match_index) = self.search_state.next_match() {
            self.list_state.select(Some(match_index));
            self.table_state.select(Some(match_index));
        }
    }

    pub fn previous_search_result(&mut self) {
        self.search_state.is_forward = false;
        if let Some(match_index) = self.search_state.next_match() {
            self.list_state.select(Some(match_index));
            self.table_state.select(Some(match_index));
        }
        self.search_state.is_forward = true;
    }

    // Progress modal methods
    pub fn start_progress(&mut self, initial_message: String) -> Sender<ProgressUpdate> {
        let (tx, rx) = mpsc::channel();
        self.progress_modal = ProgressModal {
            message: initial_message,
            percentage: 0.0,
            active: true,
        };
        self.progress_receiver = Some(rx);
        tx
    }

    pub fn update_progress(&mut self) {
        let mut should_clear_receiver = false;
        let mut updates = Vec::new();

        if let Some(ref receiver) = self.progress_receiver {
            while let Ok(update) = receiver.try_recv() {
                updates.push(update);
            }
        }

        for update in updates {
            match update {
                ProgressUpdate::Update {
                    message,
                    percentage,
                } => {
                    self.progress_modal.message = message;
                    self.progress_modal.percentage = percentage;
                }
                ProgressUpdate::Complete => {
                    self.progress_modal.active = false;
                    self.progress_modal.percentage = 100.0;
                    should_clear_receiver = true;
                    self.load_containers();
                }
                ProgressUpdate::Error(msg) => {
                    self.progress_modal.active = false;
                    self.progress_modal.message = format!("Error: {}", msg);
                    should_clear_receiver = true;
                }
            }
        }

        if should_clear_receiver {
            self.progress_receiver = None;
        }
    }

    pub fn is_progress_active(&self) -> bool {
        self.progress_modal.active
    }

    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    pub fn should_exit(&self) -> bool {
        self.exit_requested
    }

    // Toggle between running and recent containers
    pub fn toggle_recent(&mut self) {
        self.container_filter = match self.container_filter {
            ContainerFilter::Running => ContainerFilter::Recent,
            ContainerFilter::Recent => ContainerFilter::Running,
            ContainerFilter::All => ContainerFilter::Recent,
        };
        self.load_containers();
        // Reset selection
        if !self.containers.is_empty() {
            self.list_state.select(Some(0));
            self.table_state.select(Some(0));
        }
    }

    // Toggle between running and all, with special handling from recent
    pub fn toggle_all(&mut self) {
        self.container_filter = match self.container_filter {
            ContainerFilter::Running => ContainerFilter::All,
            ContainerFilter::Recent => ContainerFilter::All,
            ContainerFilter::All => ContainerFilter::Running,
        };
        self.load_containers();
        // Reset selection
        if !self.containers.is_empty() {
            self.list_state.select(Some(0));
            self.table_state.select(Some(0));
        }
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
