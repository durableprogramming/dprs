// The process_list_tabular module provides an alternative table-based rendering
// for Docker container information. It displays containers in a structured table
// format with columns for Name, Image, Status, IP Address, and Ports. This
// tabular view offers a more compact and scannable layout compared to the
// list-based display, making it easier to compare container information at a
// glance. The module uses ratatui's Table widget with proper column widths,
// headers, and row highlighting for the selected container.

use ratatui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
    Frame,
};

use crate::dprs::app::state_machine::AppState;
use crate::shared::config::Config;

pub fn render_container_table<B: Backend>(
    f: &mut Frame,
    app_state: &mut AppState,
    area: Rect,
    config: &Config,
) {
    // Define table headers
    let header_cells = ["Name", "Image", "Status", "IP Address", "Ports"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .bg(config.get_color("background_table"))
                    .fg(config.get_color("message_warning"))
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells)
        .style(Style::default().bg(config.get_color("background_table")))
        .height(1)
        .bottom_margin(1);

    // Create rows from containers
    let displayed_containers = app_state.get_displayed_containers();
    let rows = displayed_containers.iter().map(|container| {
        let cells = vec![
            Cell::from(container.name.clone()).style(
                Style::default()
                    .bg(config.get_color("background_very_dark"))
                    .fg(config.get_color("container_name")),
            ),
            Cell::from(container.image.clone()).style(
                Style::default()
                    .bg(config.get_color("background_very_dark"))
                    .fg(config.get_color("container_image_tabular")),
            ),
            Cell::from(container.status.clone()).style(
                Style::default()
                    .bg(config.get_color("background_very_dark"))
                    .fg(config.get_color("container_status_tabular")),
            ),
            Cell::from(container.ip_address.clone()).style(
                Style::default()
                    .bg(config.get_color("background_very_dark"))
                    .fg(config.get_color("container_ip_tabular")),
            ),
            Cell::from(container.ports.clone()).style(
                Style::default()
                    .bg(config.get_color("background_very_dark"))
                    .fg(config.get_color("container_ports_tabular")),
            ),
        ];
        Row::new(cells).height(1).bottom_margin(0)
    });

    // Define column widths
    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Percentage(20),
        Constraint::Percentage(15),
        Constraint::Percentage(20),
    ];

    // Create the table
    let table = Table::new(rows, widths)
        .header(header)
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
                .style(Style::new().bg(config.get_color("background_very_dark"))),
        )
        .row_highlight_style(
            Style::default()
                .bg(config.get_color("background_selection"))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app_state.table_state);
}

// #[cfg(test)]
// mod tests;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
