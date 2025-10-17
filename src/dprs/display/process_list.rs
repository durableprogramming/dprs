// The process_list module implements the core UI component for displaying Docker container information
// in a styled list format. It renders each container with details including name, image, status,
// IP address, and port mappings. The module formats this information with appropriate colors and
// styling to enhance readability, while also handling the selection state to highlight the currently
// selected container. This component forms the main interactive area of the Docker Process Manager.

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Padding},
    Frame
};

use crate::dprs::app::state_machine::AppState;
use crate::shared::config::Config;

pub fn render_container_list<B: Backend>(f: &mut Frame, app_state: &mut AppState, area: Rect, config: &Config) {
    let displayed_containers = app_state.get_displayed_containers();
    let items: Vec<ListItem> = displayed_containers
        .iter()
        .enumerate()
        .map(|(index, c)| {
            // Check if this container is visually selected
            let is_visual_selected = app_state.visual_selection
                .as_ref()
                .map(|selection| selection.is_selected(index))
                .unwrap_or(false);

            // Check if this container matches current search
            let is_search_match = app_state.search_state.matches.contains(&index);

            let mut base_style = Style::default().bg(config.get_color("background_main"));
            if is_visual_selected {
                base_style = base_style.bg(config.get_color("background_selection_orange")); // Orange background for selection
            }
            if is_search_match {
                base_style = base_style.add_modifier(Modifier::UNDERLINED);
            }
            let header = Line::from(vec![
                Span::styled(
                    &c.name,
                    base_style
                        .fg(config.get_color("container_name"))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" (", base_style),
                Span::styled(&c.image, base_style.fg(config.get_color("container_image"))),
                Span::styled(")", base_style),
            ]);

            let status = Line::from(vec![
                Span::styled("Status: ", base_style),
                Span::styled(&c.status, base_style.fg(config.get_color("container_status"))),
            ]);

            let ip = Line::from(vec![
                Span::styled("IP:     ", base_style),
                Span::styled(&c.ip_address, base_style.fg(config.get_color("container_ip"))),
            ]);

            let ports = Line::from(vec![
                Span::styled("Ports:  ", base_style),
                Span::styled(&c.ports, base_style.fg(config.get_color("container_ports"))),
            ]);

            let blank = Line::from(vec![Span::styled(" ", base_style)]);

            ListItem::new(vec![header, status, ip, ports, blank]).style(base_style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Docker Containers")
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

#[cfg(test)]
mod tests;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
