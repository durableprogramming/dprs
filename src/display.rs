use ratatui::{
    backend::{CrosstermBackend,Backend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::state_machine::AppState;
use crate::display::hotkey_bar::render_hotkey_bar;
use crate::display::process_list::render_container_list;
use crate::display::toast::ToastManager;

pub mod hotkey_bar;
pub mod process_list;
pub mod toast;
pub mod log_tabs;

pub fn draw<B: Backend>(f: &mut Frame, app_state: &mut AppState, toast_manager: &ToastManager) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),  // Hotkey bar at top
                Constraint::Min(1),     // Main container list
                Constraint::Length(3),  // Toast notification (shown conditionally)
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
