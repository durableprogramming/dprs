// The context_menu module implements a popup context menu for container and
// compose project actions. The menu is triggered by pressing '.' on a selected
// item and displays available actions based on matchers and state.

use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding},
    Frame,
};

use crate::dprs::app::state_machine::Container;
use crate::dprs::display::compose_view::ComposeProject;
use crate::shared::config::{Config, ContextMenuAction, ContextMenuMatcher};
use regex::Regex;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct ContextMenuState {
    pub active: bool,
    pub selected_index: usize,
    pub actions: Vec<ContextMenuAction>,
    pub target_container: Option<Container>,
    pub target_project: Option<ComposeProject>,
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMenuState {
    pub fn new() -> Self {
        Self {
            active: false,
            selected_index: 0,
            actions: Vec::new(),
            target_container: None,
            target_project: None,
        }
    }

    pub fn activate(&mut self, container: Option<Container>, project: Option<ComposeProject>, config: &Config) {
        self.active = true;
        self.selected_index = 0;
        self.target_container = container.clone();
        self.target_project = project.clone();

        // Filter actions based on matchers and enabled_when conditions
        self.actions = config
            .context_menu
            .actions
            .iter()
            .filter(|action| self.matches_action(action, &container, &project))
            .cloned()
            .collect();
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.selected_index = 0;
        self.actions.clear();
        self.target_container = None;
        self.target_project = None;
    }

    pub fn next(&mut self) {
        if !self.actions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.actions.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.actions.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.actions.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn get_selected_action(&self) -> Option<&ContextMenuAction> {
        self.actions.get(self.selected_index)
    }

    fn matches_action(
        &self,
        action: &ContextMenuAction,
        container: &Option<Container>,
        project: &Option<ComposeProject>,
    ) -> bool {
        // If no matchers, action is available for all
        if action.matchers.is_empty() {
            // Check enabled_when condition for general actions
            if let Some(ref condition) = action.enabled_when {
                if let Some(ref c) = container {
                    return self.check_enabled_condition(condition, c);
                }
                return false;
            }
            return true;
        }

        // Check if any matcher matches
        for matcher in &action.matchers {
            match matcher {
                ContextMenuMatcher::NamePattern { pattern } => {
                    if let Some(ref c) = container {
                        if let Ok(re) = Regex::new(pattern) {
                            if re.is_match(&c.name) {
                                // Check enabled_when condition
                                if let Some(ref condition) = action.enabled_when {
                                    return self.check_enabled_condition(condition, c);
                                }
                                return true;
                            }
                        }
                    }
                }
                ContextMenuMatcher::ImagePattern { pattern } => {
                    if let Some(ref c) = container {
                        if let Ok(re) = Regex::new(pattern) {
                            if re.is_match(&c.image) {
                                // Check enabled_when condition
                                if let Some(ref condition) = action.enabled_when {
                                    return self.check_enabled_condition(condition, c);
                                }
                                return true;
                            }
                        }
                    }
                }
                ContextMenuMatcher::LabelPattern { label, value } => {
                    if let Some(ref c) = container {
                        if self.check_container_label(&c.name, label, value.as_deref()) {
                            // Check enabled_when condition
                            if let Some(ref condition) = action.enabled_when {
                                return self.check_enabled_condition(condition, c);
                            }
                            return true;
                        }
                    }
                }
                ContextMenuMatcher::ComposeProject => {
                    if project.is_some() {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn check_enabled_condition(&self, condition: &str, container: &Container) -> bool {
        match condition {
            "running" => container.status.to_lowercase().contains("up"),
            "stopped" => !container.status.to_lowercase().contains("up"),
            _ => true,
        }
    }

    fn check_container_label(&self, container_name: &str, label: &str, expected_value: Option<&str>) -> bool {
        let output = Command::new("docker")
            .args([
                "inspect",
                "--format",
                &format!("{{{{index .Config.Labels \"{}\"}}}}", label),
                container_name,
            ])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Some(expected) = expected_value {
                    return value == expected;
                }
                return !value.is_empty();
            }
        }
        false
    }

    pub fn execute_selected_action(&self) -> Option<String> {
        if let Some(action) = self.get_selected_action() {
            let mut command = action.command.clone();

            // Replace placeholders with actual values
            if let Some(ref container) = self.target_container {
                command = command.replace("{name}", &container.name);
                command = command.replace("{image}", &container.image);
                command = command.replace("{ip}", &container.ip_address);
            }

            if let Some(ref project) = self.target_project {
                command = command.replace("{project}", &project.project_name);
                command = command.replace("{dir}", &project.working_dir);
                // For compose projects, try to find compose file
                let compose_file = format!("{}/docker-compose.yml", project.working_dir);
                command = command.replace("{compose_file}", &compose_file);
                // For service name, use first container name
                if let Some(first_container) = project.containers.first() {
                    command = command.replace("{service}", first_container);
                }
            }

            return Some(command);
        }
        None
    }
}

pub fn render_context_menu<B: Backend>(
    f: &mut Frame,
    context_menu: &ContextMenuState,
    config: &Config,
) {
    if !context_menu.active {
        return;
    }

    let area = centered_rect(70, 75, f.area());

    // Clear the background
    f.render_widget(Clear, area);

    // Build container info section if we have a target container
    let mut all_items: Vec<ListItem> = Vec::new();

    if let Some(ref container) = context_menu.target_container {
        // Add container information header
        all_items.push(ListItem::new(Line::from(vec![
            Span::styled("Container Information", Style::default().add_modifier(Modifier::BOLD).fg(config.get_color("text_highlight"))),
        ])));
        all_items.push(ListItem::new(Line::from(""))); // Separator

        // Container ID
        all_items.push(ListItem::new(Line::from(vec![
            Span::styled("  Container ID: ", Style::default().fg(config.get_color("text_dim"))),
            Span::raw(&container.container_id),
        ])));

        // Image Hash
        all_items.push(ListItem::new(Line::from(vec![
            Span::styled("  Image Hash:   ", Style::default().fg(config.get_color("text_dim"))),
            Span::raw(&container.image_hash),
        ])));

        // CPU Usage
        all_items.push(ListItem::new(Line::from(vec![
            Span::styled("  CPU Usage:    ", Style::default().fg(config.get_color("text_dim"))),
            Span::raw(&container.cpu_usage),
        ])));

        // Memory Usage
        all_items.push(ListItem::new(Line::from(vec![
            Span::styled("  Memory:       ", Style::default().fg(config.get_color("text_dim"))),
            Span::raw(&container.memory_usage),
        ])));

        // Uptime/Started At
        all_items.push(ListItem::new(Line::from(vec![
            Span::styled("  Started:      ", Style::default().fg(config.get_color("text_dim"))),
            Span::raw(format_started_time(&container.started_at)),
        ])));

        // Compose Project (if applicable)
        if let Some(ref project) = container.compose_project {
            all_items.push(ListItem::new(Line::from(vec![
                Span::styled("  Compose:      ", Style::default().fg(config.get_color("text_dim"))),
                Span::raw(project),
            ])));
        }

        all_items.push(ListItem::new(Line::from(""))); // Separator
        all_items.push(ListItem::new(Line::from(vec![
            Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD).fg(config.get_color("text_highlight"))),
        ])));
        all_items.push(ListItem::new(Line::from(""))); // Separator
    }

    // Add action items
    let action_items: Vec<ListItem> = context_menu
        .actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            let is_selected = index == context_menu.selected_index;
            let style = if is_selected {
                Style::default()
                    .bg(config.get_color("selected_bg"))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(
                    if is_selected { "▶ " } else { "  " },
                    style,
                ),
                Span::styled(&action.label, style),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    all_items.extend(action_items);

    let title = if context_menu.target_project.is_some() {
        "Context Menu - Compose Project"
    } else if context_menu.target_container.is_some() {
        "Context Menu - Container"
    } else {
        "Context Menu"
    };

    let list = List::new(all_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_type(BorderType::Rounded)
                .border_style(
                    Style::default()
                        .fg(config.get_color("border_light"))
                )
                .style(Style::default().bg(config.get_color("background_dark")))
                .padding(Padding::uniform(1)),
        );

    f.render_widget(list, area);
}

fn format_started_time(started_at: &str) -> String {
    use chrono::{DateTime, Utc};

    if let Ok(dt) = DateTime::parse_from_rfc3339(started_at) {
        let now = Utc::now();
        let duration = now.signed_duration_since(dt.with_timezone(&Utc));

        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;

        if days > 0 {
            format!("{}d {}h ago", days, hours)
        } else if hours > 0 {
            format!("{}h {}m ago", hours, minutes)
        } else {
            format!("{}m ago", minutes)
        }
    } else {
        started_at.to_string()
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
