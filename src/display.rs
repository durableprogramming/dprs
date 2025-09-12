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
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::state_machine::AppState;
use crate::display::filter_input::render_filter_input;
use crate::display::hotkey_bar::render_hotkey_bar;
use crate::display::process_list::render_container_list;
use crate::display::process_list_tabular::render_container_table;
use crate::display::toast::ToastManager;

pub mod filter_input;
pub mod hotkey_bar;
pub mod log_tabs;
pub mod process_list;
pub mod process_list_tabular;
pub mod toast;

pub fn draw<B: Backend>(f: &mut Frame, app_state: &mut AppState, toast_manager: &ToastManager) {
    let size = f.area();

    // Calculate constraints based on what needs to be shown
    let mut constraints = vec![
        Constraint::Length(3), // Hotkey bar at top
        Constraint::Min(1),    // Main container list
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
    render_hotkey_bar::<B>(f, chunks[0]);

    // Render container list (tabular or normal based on mode)
    if app_state.tabular_mode {
        render_container_table::<B>(f, app_state, chunks[1]);
    } else {
        render_container_list::<B>(f, app_state, chunks[1]);
    }

    // Render filter status or toast in bottom area
    if chunks.len() > 2 {
        if !app_state.filter_text.is_empty() {
            let filter_status = format!("Filter: {} ({} matches)", 
                app_state.filter_text, 
                app_state.get_displayed_container_count());
            let filter_widget = Paragraph::new(filter_status)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Filter Status"));

            f.render_widget(filter_widget, chunks[2]);
        } else if let Some(toast) = toast_manager.get_toast() {
            let toast_widget = Paragraph::new(toast.message.clone())
                .style(Style::default().fg(Color::White).bg(Color::Blue))
                .block(Block::default().borders(Borders::ALL).title("Notification"));

            f.render_widget(toast_widget, chunks[2]);
        }
    }

    // Render filter input overlay if in filter mode
    render_filter_input::<B>(f, app_state, size);
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
