// The dprs (Docker Process Manager) binary provides a terminal user interface
// for managing Docker containers. It implements functionality to list running
// containers, view container details, copy IP addresses, open web interfaces
// in a browser, stop containers, and refresh the container list. This file
// contains the main application loop, event handling, and UI rendering code.

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{io, io::stdout, time::Duration};

use dprs::app::actions::{copy_ip_address, open_browser, stop_container};
use dprs::app::state_machine::AppState;
use dprs::display;
use dprs::display::toast::ToastManager;

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let _ = run_app(&mut terminal);

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    // App state
    let mut app_state = AppState::new();
    app_state.load_containers();

    // Toast notification manager
    let mut toast_manager = ToastManager::new();

    // Main event loop
    loop {
        // Draw the UI
        terminal.draw(|f| display::draw::<B>(f, &mut app_state, &toast_manager))?;

        // Check if toast has expired
        toast_manager.check_expired();

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => app_state.next(),
                    KeyCode::Char('k') | KeyCode::Up => app_state.previous(),
                    KeyCode::Char('c') => match copy_ip_address(&app_state) {
                        Ok(_) => toast_manager.show("IP address copied to clipboard", 2000),
                        Err(e) => toast_manager.show(&format!("Error: {}", e), 2000),
                    },
                    KeyCode::Char('l') => match open_browser(&app_state) {
                        Ok(_) => toast_manager.show("Opening in browser", 2000),
                        Err(e) => toast_manager.show(&format!("Error: {}", e), 2000),
                    },
                    KeyCode::Char('x') => match stop_container(&mut app_state) {
                        Ok(_) => toast_manager.show("Container stopped", 2000),
                        Err(e) => toast_manager.show(&format!("Error: {}", e), 2000),
                    },
                    KeyCode::Char('r') => {
                        app_state.load_containers();
                        toast_manager.show("Containers reloaded", 1000);
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use dprs::app::actions::{copy_ip_address, open_browser, stop_container};
    use dprs::app::state_machine::{AppState, Container};
    use dprs::display::toast::ToastManager;

    use crossterm::event::KeyCode;
    use std::time::Duration;

    #[test]
    fn test_app_state_next_selection() {
        let mut app_state = AppState::new();
        app_state.containers = vec![
            Container {
                name: "container1".to_string(),
                image: "image1".to_string(),
                status: "running".to_string(),
                ip_address: "192.168.1.2".to_string(),
                ports: "80:80".to_string(),
            },
            Container {
                name: "container2".to_string(),
                image: "image2".to_string(),
                status: "running".to_string(),
                ip_address: "192.168.1.3".to_string(),
                ports: "8080:8080".to_string(),
            },
        ];

        // Initial state is index 0
        assert_eq!(app_state.list_state.selected(), Some(0));

        // Move to next container
        app_state.next();
        assert_eq!(app_state.list_state.selected(), Some(1));

        // Wrap around to first container
        app_state.next();
        assert_eq!(app_state.list_state.selected(), Some(0));
    }

    #[test]
    fn test_app_state_previous_selection() {
        let mut app_state = AppState::new();
        app_state.containers = vec![
            Container {
                name: "container1".to_string(),
                image: "image1".to_string(),
                status: "running".to_string(),
                ip_address: "192.168.1.2".to_string(),
                ports: "80:80".to_string(),
            },
            Container {
                name: "container2".to_string(),
                image: "image2".to_string(),
                status: "running".to_string(),
                ip_address: "192.168.1.3".to_string(),
                ports: "8080:8080".to_string(),
            },
        ];

        // Initial state is index 0
        assert_eq!(app_state.list_state.selected(), Some(0));

        // Move to previous wraps to last item
        app_state.previous();
        assert_eq!(app_state.list_state.selected(), Some(1));

        // Move to previous again goes to first item
        app_state.previous();
        assert_eq!(app_state.list_state.selected(), Some(0));
    }

    #[test]
    fn test_toast_manager_display_and_expiration() {
        let mut toast_manager = ToastManager::new();

        // Initially no toast
        assert!(toast_manager.get_toast().is_none());

        // Show toast
        toast_manager.show("Test message", 100);
        assert!(toast_manager.get_toast().is_some());
        assert_eq!(toast_manager.get_toast().unwrap().message, "Test message");

        // Wait for toast to expire
        std::thread::sleep(Duration::from_millis(150));
        toast_manager.check_expired();
        assert!(toast_manager.get_toast().is_none());
    }

    #[test]
    fn test_toast_manager_clear() {
        let mut toast_manager = ToastManager::new();

        // Show toast
        toast_manager.show("Test message", 5000);
        assert!(toast_manager.get_toast().is_some());

        // Clear toast
        toast_manager.clear();
        assert!(toast_manager.get_toast().is_none());
    }

    #[test]
    fn test_key_event_handling() {
        let mut app_state = AppState::new();
        let mut toast_manager = ToastManager::new();

        // Mock container data
        app_state.containers = vec![Container {
            name: "container1".to_string(),
            image: "image1".to_string(),
            status: "running".to_string(),
            ip_address: "192.168.1.2".to_string(),
            ports: "80:80".to_string(),
        }];

        // Test Down key
        handle_key_event(KeyCode::Down, &mut app_state, &mut toast_manager);
        assert_eq!(app_state.list_state.selected(), Some(0)); // No change as only one container

        // Test Up key
        handle_key_event(KeyCode::Up, &mut app_state, &mut toast_manager);
        assert_eq!(app_state.list_state.selected(), Some(0)); // No change as only one container

        // Test 'r' key for refresh
        let _containers_count = app_state.containers.len();
        handle_key_event(KeyCode::Char('r'), &mut app_state, &mut toast_manager);
        assert!(toast_manager.get_toast().is_some());
        assert_eq!(
            toast_manager.get_toast().unwrap().message,
            "Containers reloaded"
        );
    }

    // Helper function to simulate key event handling logic from the main loop
    fn handle_key_event(code: KeyCode, app_state: &mut AppState, toast_manager: &mut ToastManager) {
        match code {
            KeyCode::Char('q') => {} // Would break the loop in real code
            KeyCode::Char('j') | KeyCode::Down => app_state.next(),
            KeyCode::Char('k') | KeyCode::Up => app_state.previous(),
            KeyCode::Char('c') => match copy_ip_address(&app_state) {
                Ok(_) => toast_manager.show("IP address copied to clipboard", 2000),
                Err(e) => toast_manager.show(&format!("Error: {}", e), 2000),
            },
            KeyCode::Char('l') => match open_browser(&app_state) {
                Ok(_) => toast_manager.show("Opening in browser", 2000),
                Err(e) => toast_manager.show(&format!("Error: {}", e), 2000),
            },
            KeyCode::Char('x') => match stop_container(app_state) {
                Ok(_) => toast_manager.show("Container stopped", 2000),
                Err(e) => toast_manager.show(&format!("Error: {}", e), 2000),
            },
            KeyCode::Char('r') => {
                app_state.load_containers();
                toast_manager.show("Containers reloaded", 1000);
            }
            _ => {}
        }
    }

    #[test]
    fn test_get_selected_container() {
        let mut app_state = AppState::new();

        // Empty container list
        assert!(app_state.get_selected_container().is_none());

        // Add containers
        app_state.containers = vec![
            Container {
                name: "container1".to_string(),
                image: "image1".to_string(),
                status: "running".to_string(),
                ip_address: "192.168.1.2".to_string(),
                ports: "80:80".to_string(),
            },
            Container {
                name: "container2".to_string(),
                image: "image2".to_string(),
                status: "running".to_string(),
                ip_address: "192.168.1.3".to_string(),
                ports: "8080:8080".to_string(),
            },
        ];

        // Initial selection
        app_state.list_state.select(Some(0));
        let selected = app_state.get_selected_container();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "container1");

        // Change selection
        app_state.list_state.select(Some(1));
        let selected = app_state.get_selected_container();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "container2");
    }

    #[test]
    fn test_app_state_empty_containers() {
        let mut app_state = AppState::new();
        app_state.containers = vec![];

        // Test navigation with empty container list
        app_state.next();
        assert_eq!(app_state.list_state.selected(), Some(0));

        app_state.previous();
        assert_eq!(app_state.list_state.selected(), Some(0));

        assert!(app_state.get_selected_container().is_none());
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
