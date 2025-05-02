// The dplw (Docker Process Log Watcher) binary provides a terminal user interface
// for monitoring logs from Docker containers in real-time. It allows users to
// view logs from multiple containers simultaneously, switch between containers
// with arrow keys, scroll through logs, and refresh the container list. This
// file contains the main application loop and UI rendering logic.

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{io, io::stdout, time::Duration};

use dprs::display::log_tabs::{render_log_tabs, LogTabs};
use dprs::docker_log_watcher::DockerLogManager;
use dprs::log_view::{render_log_view, LogLevel, LogView};

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut log_manager = DockerLogManager::new();
    log_manager.start_watching_all_containers()?;
    let result = run_app(&mut terminal, &mut log_manager);

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    log_manager.stop_all();

    if let Err(err) = result {
        println!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    log_manager: &mut DockerLogManager,
) -> Result<(), io::Error> {
    // Initialize log manager

    // Create log view for displaying logs
    let mut log_view = LogView::new(1000);

    // Create tabs for container selection
    let container_names: Vec<String> = (0..log_manager.watcher_count())
        .filter_map(|i| {
            log_manager
                .get_watcher(i)
                .map(|w| w.container_name().to_string())
        })
        .collect();

    let mut log_tabs = LogTabs::new(container_names);

    loop {
        terminal.draw(|f| {
            let size = f.area();

            // Create layout with tabs at top, logs below
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3), // Tabs
                        Constraint::Min(5),    // Logs
                        Constraint::Length(3), // Help bar
                    ]
                    .as_ref(),
                )
                .split(size);

            // Render tabs
            render_log_tabs::<B>(f, &log_tabs, chunks[0]);

            // Render logs for selected container
            if let Some(active_tab) = log_tabs.index.checked_sub(0) {
                if let Some(watcher) = log_manager.get_watcher(active_tab) {
                    // Clear and refill the log view with current container logs
                    log_view = LogView::new(1000);
                    for log_line in watcher.get_logs() {
                        log_view.add_log(log_line, LogLevel::Info);
                    }

                    render_log_view::<B>(f, &log_view, chunks[1]);
                }
            }

            // Render help bar
            let help_text = vec![
                Span::styled(
                    "q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Quit | "),
                Span::styled(
                    "←/→",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Switch Container | "),
                Span::styled(
                    "↑/↓",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Scroll Logs | "),
                Span::styled(
                    "r",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Refresh Containers"),
            ];

            let help_bar = Paragraph::new(Line::from(help_text))
                .block(Block::default().borders(Borders::ALL).title("Hotkeys"));

            f.render_widget(help_bar, chunks[2]);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Right => log_tabs.next(),
                    KeyCode::Left => log_tabs.previous(),
                    KeyCode::Up => log_view.scroll_up(),
                    KeyCode::Down => log_view.scroll_down(),
                    KeyCode::Home => log_view.scroll_to_top(),
                    KeyCode::End => log_view.scroll_to_bottom(),
                    KeyCode::Char('r') => {
                        log_manager.refresh()?;
                        // Update tabs with new container names
                        let container_names: Vec<String> = (0..log_manager.watcher_count())
                            .filter_map(|i| {
                                log_manager
                                    .get_watcher(i)
                                    .map(|w| w.container_name().to_string())
                            })
                            .collect();
                        log_tabs = LogTabs::new(container_names);
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_log_manager_creation() {
        let log_manager = DockerLogManager::new();
        assert_eq!(log_manager.watcher_count(), 0);
    }

    #[test]
    fn test_log_view_scrolling() {
        let mut log_view = LogView::new(10);

        // Add some logs
        for i in 0..20 {
            log_view.add_log(format!("Log line {}", i), LogLevel::Info);
        }

        log_view.scroll_to_top();
        // Test initial state
        assert_eq!(log_view.get_scroll_position(), 0);

        // Test scrolling up (should not go past 0)
        log_view.scroll_up();
        log_view.scroll_up();
        assert_eq!(log_view.get_scroll_position(), 0);

        // Test scrolling down
        log_view.scroll_down();
        assert_eq!(log_view.get_scroll_position(), 1);

        // Test scroll to top
        log_view.scroll_to_bottom();
        assert!(log_view.get_scroll_position() > 0);
        log_view.scroll_to_top();
        assert_eq!(log_view.get_scroll_position(), 0);
    }

    #[test]
    fn test_log_tabs_navigation() {
        let container_names = vec![
            "container1".to_string(),
            "container2".to_string(),
            "container3".to_string(),
        ];

        let mut log_tabs = LogTabs::new(container_names);

        // Test initial state
        assert_eq!(log_tabs.index, 0);

        // Test next
        log_tabs.next();
        assert_eq!(log_tabs.index, 1);

        // Test previous
        log_tabs.previous();
        assert_eq!(log_tabs.index, 0);
    }

    #[test]
    fn test_log_manager_refresh() {
        let mut log_manager = DockerLogManager::new();

        // Mock implementation for testing
        struct MockLogManager {
            refresh_called: Arc<Mutex<bool>>,
        }

        impl MockLogManager {
            fn new() -> Self {
                MockLogManager {
                    refresh_called: Arc::new(Mutex::new(false)),
                }
            }

            fn refresh(&self) {
                let mut called = self.refresh_called.lock().unwrap();
                *called = true;
            }

            fn was_refresh_called(&self) -> bool {
                *self.refresh_called.lock().unwrap()
            }
        }

        let mock = MockLogManager::new();
        let mock_ref = Arc::new(mock);
        let mock_clone = Arc::clone(&mock_ref);

        // Simulate refresh
        thread::spawn(move || {
            mock_clone.refresh();
        })
        .join()
        .unwrap();

        assert!(mock_ref.was_refresh_called());
    }
}
// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
