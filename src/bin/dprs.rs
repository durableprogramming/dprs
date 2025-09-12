// The dprs (Docker Process Manager) binary provides a terminal user interface
// for managing Docker containers. It allows users to list running containers,
// view details, stop containers, copy IP addresses, and open web interfaces.
// This file contains the main application loop and UI rendering logic for dprs.

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use dprs::app::{actions, AppState};
use dprs::display;
use dprs::display::toast::ToastManager;

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // Create app state and toast manager
    let mut toast_manager = ToastManager::new();


    let result = run_app(&mut terminal,  &mut toast_manager);

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    if let Err(err) = result {
        // Print errors that occur during the TUI loop itself.
        println!("Application error: {}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    toast_manager: &mut ToastManager,
) -> Result<(), io::Error> {
    let mut last_refresh = Instant::now();
    let refresh_interval = Duration::from_millis(500); // Refresh every half second
    let mut app_state = AppState::new();

    // Initial load of containers
    if let Err(e) = app_state.refresh_containers() {
        // This error occurs before the TUI loop starts, so print to stderr.
        // The TUI will still attempt to start and periodic refreshes will occur.
        eprintln!(
            "Initial container load failed: {}. The list may be empty or stale.",
            e
        );
    }

    loop {
        // Draw UI
        terminal.draw(|f| display::draw::<B>(f, &mut app_state, &toast_manager))?;

        // Handle toast expiration
        toast_manager.check_expired();

        // Determine polling timeout: either time until next refresh or a small default
        let time_since_last_refresh = last_refresh.elapsed();
        let poll_timeout = if time_since_last_refresh >= refresh_interval {
            Duration::from_millis(0) // Refresh is due, poll non-blockingly
        } else {
            refresh_interval - time_since_last_refresh // Time remaining until next refresh
        };

        // Handle input events
        if event::poll(poll_timeout)? {
            if let Event::Key(key) = event::read()? {
                // Handle filter mode input
                if app_state.filter_mode {
                    match key.code {
                        KeyCode::Enter => {
                            app_state.exit_filter_mode();
                        }
                        KeyCode::Esc => {
                            app_state.exit_filter_mode();
                            app_state.clear_filter();
                        }
                        KeyCode::Backspace => {
                            let mut text = app_state.filter_text.clone();
                            text.pop();
                            app_state.update_filter(text);
                        }
                        KeyCode::Char(c) => {
                            let mut text = app_state.filter_text.clone();
                            text.push(c);
                            app_state.update_filter(text);
                        }
                        _ => {}
                    }
                    continue;
                }

                // Handle normal mode input
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => app_state.next(),
                    KeyCode::Char('k') | KeyCode::Up => app_state.previous(),
                    KeyCode::Char('c') => match actions::copy_ip_address(&mut app_state) {
                        Ok(_) => toast_manager.show("IP address copied to clipboard!", 2000),
                        Err(e) => toast_manager.show(&format!("Error copying IP: {}", e), 3000),
                    },
                    KeyCode::Char('l') => match actions::open_browser(&mut app_state) {
                        Ok(_) => toast_manager.show("Opening browser...", 2000),
                        Err(e) => toast_manager.show(&format!("Error opening browser: {}", e), 3000),
                    },
                    KeyCode::Char('x') => match actions::stop_container(&mut app_state) {
                        Ok(_) => {
                            toast_manager.show("Stop command sent. Refreshing list...", 2000);
                            // stop_container already calls load_containers
                        }
                        Err(e) => toast_manager.show(&format!("Error stopping container: {}", e), 3000),
                    },
                    KeyCode::Char('r') => { // Restart container
                        match actions::restart_container(&mut app_state) {
                            Ok(_) => {
                                toast_manager.show("Restart command sent. Refreshing list...", 2000);
                                // restart_container already calls load_containers
                            }
                            Err(e) => toast_manager.show(&format!("Error restarting container: {}", e), 3000),
                        }
                    }
                    KeyCode::F(5) => { // Refresh container list
                        match app_state.refresh_containers() {
                            Ok(_) => toast_manager.show("Container list refreshed.", 2000),
                            Err(e) => toast_manager.show(&format!("Error refreshing containers: {}", e), 3000),
                        }
                        last_refresh = Instant::now(); // Reset timer after manual refresh
                    }
                    KeyCode::Char('t') => { // Toggle tabular view
                        app_state.tabular_mode = !app_state.tabular_mode;
                        let mode_text = if app_state.tabular_mode { "tabular" } else { "normal" };
                        toast_manager.show(&format!("Switched to {} view", mode_text), 1500);
                    }
                    KeyCode::Char('/') => { // Enter filter mode
                        app_state.enter_filter_mode();
                    }
                    KeyCode::Esc => { // Clear filter when not in filter mode
                        if !app_state.filter_text.is_empty() {
                            app_state.clear_filter();
                            toast_manager.show("Filter cleared", 1500);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Periodic refresh check (if poll timed out or no event handled that resets the timer)
        if last_refresh.elapsed() >= refresh_interval {
            if let Err(e) = app_state.refresh_containers() {
                toast_manager.show(&format!("Auto-refresh error: {}", e), 3000);
            }
            // No toast for successful auto-refresh to avoid being too noisy.
            last_refresh = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn basic_startup_test() {
        let result = std::panic::catch_unwind(|| {
            let _app_state = dprs::app::AppState::new();
            let _toast_manager = dprs::display::toast::ToastManager::new();
        });
        assert!(result.is_ok());
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
