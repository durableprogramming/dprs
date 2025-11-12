// The compose_view module implements a view for displaying and managing Docker Compose projects.
// It groups containers by their compose project using the com.docker.compose.project.working_dir label,
// allowing project-level operations like restarting or stopping entire projects at once.

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Padding},
    Frame,
};

use crate::dprs::app::state_machine::AppState;
use crate::shared::config::Config;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ComposeProject {
    pub project_name: String,
    pub working_dir: String,
    pub containers: Vec<String>,
}

impl ComposeProject {
    pub fn container_count(&self) -> usize {
        self.containers.len()
    }
}

pub fn group_containers_by_project(app_state: &AppState) -> Vec<ComposeProject> {
    use std::process::Command;

    let mut projects: HashMap<String, ComposeProject> = HashMap::new();

    for container in &app_state.containers {
        // Get compose project labels from docker inspect
        let output = Command::new("docker")
            .args([
                "inspect",
                "--format",
                "{{index .Config.Labels \"com.docker.compose.project.working_dir\"}}|{{index .Config.Labels \"com.docker.compose.project\"}}",
                &container.name,
            ])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = output_str.trim().split('|').collect();

                if parts.len() == 2 {
                    let working_dir = parts[0].to_string();
                    let project_name = parts[1].to_string();

                    // Only include if both labels exist (not empty)
                    if !working_dir.is_empty() && !project_name.is_empty() {
                        projects
                            .entry(working_dir.clone())
                            .or_insert_with(|| ComposeProject {
                                project_name: project_name.clone(),
                                working_dir: working_dir.clone(),
                                containers: Vec::new(),
                            })
                            .containers
                            .push(container.name.clone());
                    }
                }
            }
        }
    }

    let mut project_list: Vec<ComposeProject> = projects.into_values().collect();
    project_list.sort_by(|a, b| a.project_name.cmp(&b.project_name));
    project_list
}

pub fn render_compose_view<B: Backend>(
    f: &mut Frame,
    app_state: &mut AppState,
    area: Rect,
    config: &Config,
) {
    let projects = group_containers_by_project(app_state);

    let items: Vec<ListItem> = projects
        .iter()
        .enumerate()
        .map(|(index, project)| {
            // Check if this project is visually selected
            let is_visual_selected = app_state
                .visual_selection
                .as_ref()
                .map(|selection| selection.is_selected(index))
                .unwrap_or(false);

            let mut base_style = Style::default().bg(config.get_color("background_main"));
            if is_visual_selected {
                base_style = base_style.bg(config.get_color("background_selection_orange"));
            }

            let header = Line::from(vec![
                Span::styled(
                    &project.project_name,
                    base_style
                        .fg(config.get_color("container_name"))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" (", base_style),
                Span::styled(
                    format!("{} containers", project.container_count()),
                    base_style.fg(config.get_color("container_image")),
                ),
                Span::styled(")", base_style),
            ]);

            let working_dir = Line::from(vec![
                Span::styled("Dir:    ", base_style),
                Span::styled(
                    &project.working_dir,
                    base_style.fg(config.get_color("container_status")),
                ),
            ]);

            let containers_label = Line::from(vec![Span::styled(
                format!("Services: {}", project.containers.join(", ")),
                base_style.fg(config.get_color("container_ip")),
            )]);

            let blank = Line::from(vec![Span::styled(" ", base_style)]);

            ListItem::new(vec![header, working_dir, containers_label, blank]).style(base_style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Docker Compose Projects")
                .border_type(BorderType::Rounded)
                .border_style(
                    Style::default()
                        .bg(config.get_color("background_alt_dark"))
                        .fg(config.get_color("text_selection")),
                )
                .style(Style::new().bg(config.get_color("background_very_dark")))
                .padding(Padding::vertical(1)),
        )
        .highlight_style(
            Style::default()
                .bg(config.get_color("selected_bg"))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app_state.list_state);
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
