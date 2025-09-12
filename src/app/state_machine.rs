// The state_machine module manages the core application state for the
// Docker container management TUI. It defines the Container struct to store
// container information, AppEvent enum for user interactions, and AppState
// for maintaining the current application state including container list
// and selection. The implementation includes methods to navigate container
// lists, select containers, and refresh container data by querying the
// Docker CLI. This serves as the central data model for the application.

use ratatui::widgets::{ListState, TableState};
use std::io::{Error, ErrorKind};
use std::process::Command;

#[derive(Clone)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub status: String,
    pub ip_address: String,
    pub ports: String,
}

pub enum AppEvent {
    SelectNext,
    SelectPrevious,
    Refresh,
    CopyIp,
    OpenBrowser,
    StopContainer,
    Quit,
    EnterFilterMode,
    ExitFilterMode,
    ClearFilter,
}

pub struct AppState {
    pub containers: Vec<Container>,
    pub list_state: ListState,
    pub table_state: TableState,
    pub tabular_mode: bool,
    pub filter_mode: bool,
    pub filter_text: String,
    pub filtered_containers: Vec<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
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
            filter_mode: false,
            filter_text: String::new(),
            filtered_containers: Vec::new(),
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
                    self.filtered_containers.get(i)
                        .and_then(|&real_index| self.containers.get(real_index))
                }
            }
            None => None,
        }
    }

    pub fn refresh_containers(&mut self) -> Result<(), Error> {
        self.containers.clear();

        let output = Command::new("docker")
            .args([
                "ps",
                "--format",
                "{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}",
            ])
            .output()
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to execute docker command: {}", e),
                )
            })?;

        if !output.status.success() {
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Docker command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
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
                    .args([
                        "inspect",
                        "--format",
                        "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}",
                        &name,
                    ])
                    .output()
                    .map_err(|e| {
                        Error::new(
                            ErrorKind::Other,
                            format!("Failed to get container IP: {}", e),
                        )
                    })?;

                let ip_address = String::from_utf8_lossy(&ip_output.stdout)
                    .trim()
                    .to_string();

                self.containers.push(Container {
                    name,
                    image,
                    status,
                    ip_address,
                    ports,
                });
            }
        }

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
        self.filtered_containers = self.containers
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
        if self.filter_text.is_empty() {
            self.containers.len()
        } else {
            self.filtered_containers.len()
        }
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
