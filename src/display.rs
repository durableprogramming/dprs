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
use crate::display::hotkey_bar::render_hotkey_bar;
use crate::display::process_list::render_container_list;
use crate::display::toast::ToastManager;

pub mod hotkey_bar;
pub mod log_tabs;
pub mod process_list;
pub mod toast;

pub fn draw<B: Backend>(f: &mut Frame, app_state: &mut AppState, toast_manager: &ToastManager) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Hotkey bar at top
                Constraint::Min(1),    // Main container list
                Constraint::Length(3), // Toast notification (shown conditionally)
            ]
            .as_ref(),
        )
        .split(size);

    // Render the hotkey bar
    render_hotkey_bar::<B>(f, chunks[0]);

    // Render container list
    render_container_list::<B>(f, app_state, chunks[1]);

    // Render toast if available
    if let Some(toast) = toast_manager.get_toast() {
        let toast_widget = Paragraph::new(toast.message.clone())
            .style(Style::default().fg(Color::White).bg(Color::Blue))
            .block(Block::default().borders(Borders::ALL).title("Notification"));

        f.render_widget(toast_widget, chunks[2]);
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
