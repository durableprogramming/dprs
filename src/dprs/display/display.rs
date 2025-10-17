// The display module provides UI rendering components for the Docker Process Management TUI.
// It defines the main draw function that orchestrates rendering of all UI elements
//
// and contains submodules for specific interface components:
//
// - hotkey_bar: displays available keyboard shortcuts at the top of the screen
// - process_list: renders the container list with details like name, status, and IP
// - toast: implements a notification system for user feedback
// - log_tabs: provides container selection tabs for the log viewer
//
// The module creates a consistent visual layout and handles all rendering logic.

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tachyonfx::{CellFilter, Duration, EffectManager, Interpolation};

use crate::dprs::app::state_machine::AppState;
use crate::dprs::display::filter_input::render_filter_input;
use crate::dprs::display::hotkey_bar::render_hotkey_bar;
use crate::dprs::display::process_list::render_container_list;
use crate::dprs::display::process_list_tabular::render_container_table;
use crate::dprs::display::toast::ToastManager;
use crate::dprs::modes::Mode;
use crate::shared::config::Config;

pub fn draw<B: Backend>(
    f: &mut Frame,
    app_state: &mut AppState,
    toast_manager: &ToastManager,
    config: &mut Config,
    effects: &mut EffectManager<()>,
    elapsed: std::time::Duration,
) {
    let size = f.area();

    // Clear the entire frame with the main background color
    let background =
        Block::default().style(Style::default().bg(config.get_color("background_main")));
    f.render_widget(background, size);

    // Calculate constraints based on what needs to be shown
    let mut constraints = vec![
        Constraint::Length(3), // Hotkey bar at top
        Constraint::Min(1),    // Main container list
        Constraint::Length(3), // Status line at bottom
    ];

    // Add constraint for filter status or toast
    if !app_state.filter_text.is_empty() || toast_manager.get_toast().is_some() {
        constraints.push(Constraint::Length(3));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(size);

    // Render the hotkey bar
    render_hotkey_bar::<B>(f, chunks[0], &*config);

    // Render container list (tabular or normal based on mode)
    let container_area = if app_state.tabular_mode {
        render_container_table::<B>(f, app_state, chunks[1], &*config);
        chunks[1]
    } else {
        render_container_list::<B>(f, app_state, chunks[1], &*config);
        chunks[1]
    };

    // Add swipe-in effects for new containers (skip if progress modal is active)
    if !app_state.is_progress_active() {
        add_container_effects(f, app_state, effects, container_area, elapsed, config);
    }

    // Render status line
    render_status_line(f, app_state, chunks[2], &*config);

    // Render filter status or toast in additional bottom area
    if chunks.len() > 3 {
        if !app_state.filter_text.is_empty() {
            let filter_status = format!(
                "Filter: {} ({} matches)",
                app_state.filter_text,
                app_state.get_displayed_container_count()
            );
            let filter_widget = Paragraph::new(filter_status)
                .style(
                    Style::default()
                        .fg(config.get_color("filter_text"))
                        .bg(config.get_color("background_dark")),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Filter Status"),
                );

            f.render_widget(filter_widget, chunks[3]);
        } else if let Some(toast) = toast_manager.get_toast() {
            let toast_widget = Paragraph::new(toast.message.clone())
                .style(
                    Style::default()
                        .fg(config.get_color("text_main"))
                        .bg(config.get_color("mode_command")),
                )
                .block(Block::default().borders(Borders::ALL).title("Notification"));

            f.render_widget(toast_widget, chunks[3]);
        }
    }

    // Render filter input overlay if in filter mode
    render_filter_input::<B>(f, app_state, size, &*config);

    // Render command line overlay if in command or search mode
    render_command_line(f, app_state, size, &*config);

    // Render progress modal if active and experimental animation flag is set
    if app_state.is_progress_active() && config.general.experimental_fx {
        render_progress_modal(f, app_state, size, &*config, effects, elapsed);
    }
}

fn render_status_line(
    f: &mut Frame,
    app_state: &AppState,
    area: ratatui::layout::Rect,
    config: &Config,
) {
    let mut status_parts = Vec::new();

    // Mode indicator
    let mode_style = match app_state.mode {
        Mode::Normal => Style::default().fg(config.get_color("mode_normal")),
        Mode::Visual => Style::default().fg(config.get_color("mode_visual")),
        Mode::Command => Style::default().fg(config.get_color("mode_command")),
        Mode::Search => Style::default().fg(config.get_color("mode_search")),
    };

    status_parts.push(format!("-- {} --", app_state.mode.display_name()));

    // Container info
    if let Some(container) = app_state.get_selected_container() {
        status_parts.push(format!(
            "Container: {} ({})",
            container.name, container.status
        ));
        if !container.ip_address.is_empty() {
            status_parts.push(format!("IP: {}", container.ip_address));
        }
    }

    // Position indicator
    let total_count = app_state.get_displayed_container_count();
    if let Some(selected) = app_state.list_state.selected() {
        status_parts.push(format!("{}/{}", selected + 1, total_count));
    }

    // Search results
    if !app_state.search_state.matches.is_empty() {
        let matches_count = app_state.search_state.matches.len();
        let current_match = app_state
            .search_state
            .current_match
            .map(|i| i + 1)
            .unwrap_or(0);
        status_parts.push(format!(
            "Search: {}/{} matches",
            current_match, matches_count
        ));
    }

    // Visual selection info
    if let Some(selection) = &app_state.visual_selection {
        status_parts.push(format!(
            "Selected: {} containers",
            selection.selected_indices.len()
        ));
    }

    let status_text = status_parts.join(" | ");
    let status_widget = Paragraph::new(status_text)
        .style(mode_style.bg(config.get_color("background_dark")))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::new().bg(config.get_color("background_dark"))),
        );

    f.render_widget(status_widget, area);
}

fn render_command_line(
    f: &mut Frame,
    app_state: &AppState,
    area: ratatui::layout::Rect,
    config: &Config,
) {
    match app_state.mode {
        Mode::Command => {
            let input_text = format!(":{}", app_state.command_state.input);
            let command_widget = Paragraph::new(input_text)
                .style(
                    Style::default()
                        .fg(config.get_color("text_main"))
                        .bg(config.get_color("mode_command")),
                )
                .block(Block::default().borders(Borders::ALL).title("Command"));

            let popup_area = ratatui::layout::Rect {
                x: 0,
                y: area.height.saturating_sub(3),
                width: area.width,
                height: 3,
            };

            f.render_widget(command_widget, popup_area);
        }
        Mode::Search => {
            let search_prefix = if app_state.search_state.is_forward {
                "/"
            } else {
                "?"
            };
            let input_text = format!("{}{}", search_prefix, app_state.search_state.query);
            let search_widget = Paragraph::new(input_text)
                .style(
                    Style::default()
                        .fg(config.get_color("text_main"))
                        .bg(config.get_color("mode_search")),
                )
                .block(Block::default().borders(Borders::ALL).title("Search"));

            let popup_area = ratatui::layout::Rect {
                x: 0,
                y: area.height.saturating_sub(3),
                width: area.width,
                height: 3,
            };

            f.render_widget(search_widget, popup_area);
        }
        _ => {}
    }
}

fn render_progress_modal(
    f: &mut Frame,
    app_state: &AppState,
    area: Rect,
    config: &Config,
    effects: &mut EffectManager<()>,
    elapsed: std::time::Duration,
) {
    let modal_width = 70;
    let modal_height = 10;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect {
        x,
        y,
        width: modal_width,
        height: modal_height,
    };

    // Clear the entire screen with a dark overlay to prevent background bleed
    let overlay = Block::default().style(Style::default().bg(config.get_color("background_main")));
    f.render_widget(overlay, area);

    // Force clear the buffer in the modal area to prevent ghosting
    for y_pos in modal_area.y..modal_area.y + modal_area.height {
        for x_pos in modal_area.x..modal_area.x + modal_area.width {
            if let Some(cell) = f.buffer_mut().cell_mut((x_pos, y_pos)) {
                cell.set_char(' ');
                cell.set_bg(config.get_color("background_dark"));
            }
        }
    }

    // Create inner layout with padding
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title area
            Constraint::Length(3), // Gauge area
            Constraint::Length(2), // Status text area
            Constraint::Min(0),    // Bottom padding
        ])
        .split(modal_area);

    // Render modal background with border
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(config.get_color("border_main")))
        .style(Style::default().bg(config.get_color("background_dark")));
    f.render_widget(modal_block, modal_area);

    // Title bar
    let title = Paragraph::new("Operation in Progress")
        .style(
            Style::default()
                .fg(config.get_color("border_main"))
                .bg(config.get_color("background_dark")),
        )
        .block(Block::default());
    f.render_widget(title, inner_chunks[0]);

    // Add horizontal padding to gauge area
    let gauge_area = Rect {
        x: inner_chunks[1].x + 2,
        y: inner_chunks[1].y,
        width: inner_chunks[1].width.saturating_sub(4),
        height: inner_chunks[1].height,
    };

    // Render indeterminate progress bar background
    let progress_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(config.get_color("border_light")))
        .style(Style::default().bg(config.get_color("background_alt")));
    f.render_widget(progress_block, gauge_area);

    // Status message with padding
    let status_area = Rect {
        x: inner_chunks[2].x + 2,
        y: inner_chunks[2].y,
        width: inner_chunks[2].width.saturating_sub(4),
        height: inner_chunks[2].height,
    };
    let status_msg = Paragraph::new(app_state.progress_modal.message.as_str())
        .style(
            Style::default()
                .fg(config.get_color("text_main"))
                .bg(config.get_color("background_dark")),
        )
        .block(Block::default());
    f.render_widget(status_msg, status_area);

    // Add animated effect to the progress bar
    use tachyonfx::fx;
    let filter = CellFilter::Area(gauge_area);
    let effect = fx::ping_pong(
        fx::hsl_shift(
            None,
            Some([0.0, 0.0, 10.0]),
            (Duration::from_millis(1000), Interpolation::SineInOut),
        )
        .with_filter(filter),
    );
    effects.add_effect(effect);

    // Process effects
    effects.process_effects(elapsed.into(), f.buffer_mut(), gauge_area);
}

fn add_container_effects(
    f: &mut Frame,
    app_state: &mut AppState,
    effects: &mut EffectManager<()>,
    container_area: Rect,
    elapsed: std::time::Duration,
    config: &Config,
) {
    use tachyonfx::fx;

    // Only add effects if experimental_fx is enabled
    if config.general.experimental_fx {
        // Add swipe-in effects for new containers
        for &index in &app_state.new_container_indices {
            // Calculate the row range for this container item
            // Assuming each container takes about 5 lines (header + 4 details)
            let item_start_row = container_area.y + (index as u16 * 5);
            let item_end_row = item_start_row + 4; // 5 lines total

            if item_end_row < container_area.y + container_area.height {
                let item_area = Rect::new(
                    container_area.x,
                    item_start_row,
                    container_area.width,
                    item_end_row - item_start_row + 1,
                );
                let filter = CellFilter::Area(item_area);

                let effect = fx::coalesce((Duration::from_millis(250), Interpolation::QuadOut))
                    .with_filter(filter);
                effects.add_effect(effect);
            }
        }

        // Process effects
        effects.process_effects(elapsed.into(), f.buffer_mut(), container_area);
    }

    // Clear the new container indices after adding effects
    app_state.new_container_indices.clear();
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
