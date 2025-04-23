use std::{io, time::Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;

mod app;
mod display;

use app::state_machine::AppState;
use display::{hotkey_bar, process_list, toast::ToastManager};

fn main() -> Result<(), io::Error> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Application state
    let mut app_state = AppState::new();
    let mut toast_manager = ToastManager::new();

    // Load initial data
    app_state.refresh_containers()?;

    // Main event loop
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(3),
                ].as_ref())
                .split(f.size());

            // Render container list
            process_list::render_container_list(f, &mut app_state, chunks[0]);
            
            // Render hotkey bar
            hotkey_bar::render_hotkey_bar(f, chunks[1]);
            
            // TODO: Render toast notifications if implemented
        })?;

        // Check for toast expiration
        toast_manager.check_expired();

        // Check for events with timeout
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => app_state.next(),
                    KeyCode::Char('k') | KeyCode::Up => app_state.previous(),
                    KeyCode::Char('c') => {
                        // Copy IP address to clipboard
                        if let Some(container) = app_state.get_selected_container() {
                            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                            ctx.set_contents(container.ip_address.clone()).unwrap();
                            toast_manager.show("IP address copied to clipboard!", 2000);
                        }
                    },
                    KeyCode::Char('l') => {
                        // Open in browser
                        if let Some(container) = app_state.get_selected_container() {
                            // TODO: Implement browser opening functionality
                            toast_manager.show(&format!("Opening {} in browser", container.name), 2000);
                        }
                    },
                    KeyCode::Char('x') => {
                        // Stop container
                        if let Some(container) = app_state.get_selected_container() {
                            // TODO: Implement container stopping functionality
                            toast_manager.show(&format!("Stopping container {}", container.name), 2000);
                        }
                    },
                    KeyCode::Char('r') => {
                        // Refresh container list
                        app_state.refresh_containers()?;
                        toast_manager.show("Container list refreshed", 2000);
                    },
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
            // Perform periodic updates here if needed
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
