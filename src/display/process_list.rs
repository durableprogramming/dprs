// The process_list module implements the core UI component for displaying Docker container information
// in a styled list format. It renders each container with details including name, image, status,
// IP address, and port mappings. The module formats this information with appropriate colors and
// styling to enhance readability, while also handling the selection state to highlight the currently
// selected container. This component forms the main interactive area of the Docker Process Manager.

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Padding},
    Frame,
};

use crate::app::state_machine::AppState;

pub fn render_container_list<B: Backend>(f: &mut Frame, app_state: &mut AppState, area: Rect) {
    let containers = &app_state.containers;

    let items: Vec<ListItem> = containers
        .iter()
        .map(|c| {
            let header = Line::from(vec![
                Span::styled(
                    &c.name,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" ("),
                Span::styled(&c.image, Style::default().fg(Color::Yellow)),
                Span::raw(")"),
            ]);

            let status = Line::from(vec![
                Span::raw("Status: "),
                Span::styled(&c.status, Style::default().fg(Color::Cyan)),
            ]);

            let ip = Line::from(vec![
                Span::raw("IP:     "),
                Span::styled(&c.ip_address, Style::default().fg(Color::Blue)),
            ]);

            let ports = Line::from(vec![
                Span::raw("Ports:  "),
                Span::styled(&c.ports, Style::default().fg(Color::Magenta)),
            ]);

            let blank = Line::from(vec![Span::raw(" ")]);

            ListItem::new(vec![header, status, ip, ports, blank]).style(Style::default())
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
                        .bg(Color::Rgb(15, 32, 48))
                        .fg(Color::Rgb(128, 128, 255)),
                )
                .style(Style::new().bg(Color::Rgb(8, 8, 32)))
                .padding(Padding::vertical(1)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(15, 32, 48))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app_state.list_state);
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
