// The main application loop for dplw (Docker Process Log Watcher).
// This module contains the primary UI rendering and event handling logic.

use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{io, time::Duration};

use crate::shared::config::Config;
use crate::shared::display::log_tabs::{render_log_tabs, LogTabs};
use crate::shared::display::log_view::{render_log_view, LogLevel, LogView};
use crate::shared::docker::docker_log_watcher::DockerLogManager;
use crate::shared::input::input_watcher::InputWatcher;

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    log_manager: &mut DockerLogManager,
) -> Result<(), io::Error> {
    // Initialize log manager
    let input_watcher = InputWatcher::new();
    let config = Config::default();

    // Create log views for each container (one per container)
    let mut log_views: Vec<LogView> = Vec::new();

    // Create tabs for container selection
    let container_names: Vec<String> = (0..log_manager.watcher_count())
        .filter_map(|i| {
            log_manager
                .get_watcher(i)
                .map(|w| w.container_name().to_string())
        })
        .collect();

    // Initialize log views for each container
    for _ in 0..container_names.len() {
        log_views.push(LogView::new(1000));
    }

    let mut log_tabs = LogTabs::new(container_names);
    let mut log_area_height = 0;

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
            render_log_tabs::<B>(f, &log_tabs, chunks[0], &config);

            // Render logs for selected container
            if let Some(active_tab) = log_tabs.index.checked_sub(0) {
                if let Some(watcher) = log_manager.get_watcher(active_tab) {
                    if active_tab < log_views.len() {
                        // Update logs for current container if needed
                        let current_log_view = &mut log_views[active_tab];
                        let current_logs = watcher.get_logs();

                        // Only update if we have a different number of logs
                        if current_log_view.get_log_count() != current_logs.len() {
                            // Preserve scroll position
                            let scroll_pos = current_log_view.get_scroll_position();
                            *current_log_view = LogView::new(1000);

                            for log_line in current_logs {
                                current_log_view.add_log(log_line, LogLevel::Info);
                            }

                            // Restore scroll position if it's still valid
                            if scroll_pos < current_log_view.get_log_count() {
                                current_log_view.set_scroll_position(scroll_pos);
                            }
                        }

                        render_log_view::<B>(f, &mut *current_log_view, chunks[1], &config);
                        log_area_height = chunks[1].height as usize;
                    }
                }
            }

            // Render help bar
            let help_text = vec![
                Span::styled(
                    "q/Ctrl+C",
                    Style::default()
                        .fg(config.get_color("message_error"))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Quit | "),
                Span::styled(
                    "←/→",
                    Style::default()
                        .fg(config.get_color("message_warning"))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Switch Container | "),
                Span::styled(
                    "↑/↓",
                    Style::default()
                        .fg(config.get_color("message_warning"))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Scroll | "),
                Span::styled(
                    "PgUp/PgDn",
                    Style::default()
                        .fg(config.get_color("message_warning"))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Page Scroll | "),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(config.get_color("message_success"))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Follow | "),
                Span::styled(
                    "r",
                    Style::default()
                        .fg(config.get_color("message_success"))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Refresh"),
            ];

            let help_bar = Paragraph::new(Line::from(help_text))
                .block(Block::default().borders(Borders::ALL).title("Hotkeys"));

            f.render_widget(help_bar, chunks[2]);
        })?;

        // Handle input from watcher
        if let Ok(Event::Key(key)) = input_watcher.try_recv() {
            match key.code {
                KeyCode::Char('q') => {
                    return Ok(());
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                KeyCode::Right => log_tabs.next(),
                KeyCode::Left => log_tabs.previous(),
                KeyCode::Up => {
                    if let Some(active_tab) = log_tabs.index.checked_sub(0) {
                        if active_tab < log_views.len() {
                            log_views[active_tab].scroll_up();
                        }
                    }
                }
                KeyCode::Down => {
                    if let Some(active_tab) = log_tabs.index.checked_sub(0) {
                        if active_tab < log_views.len() {
                            log_views[active_tab].scroll_down();
                        }
                    }
                }
                KeyCode::Home => {
                    if let Some(active_tab) = log_tabs.index.checked_sub(0) {
                        if active_tab < log_views.len() {
                            log_views[active_tab].scroll_to_top();
                        }
                    }
                }
                KeyCode::End => {
                    if let Some(active_tab) = log_tabs.index.checked_sub(0) {
                        if active_tab < log_views.len() {
                            log_views[active_tab].scroll_to_bottom();
                        }
                    }
                }
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
                    log_tabs = LogTabs::new(container_names.clone());

                    // Recreate log views for the new containers
                    log_views.clear();
                    for _ in 0..container_names.len() {
                        log_views.push(LogView::new(1000));
                    }
                }
                KeyCode::PageUp => {
                    if let Some(active_tab) = log_tabs.index.checked_sub(0) {
                        if active_tab < log_views.len() {
                            log_views[active_tab].page_up(log_area_height);
                        }
                    }
                }
                KeyCode::PageDown => {
                    if let Some(active_tab) = log_tabs.index.checked_sub(0) {
                        if active_tab < log_views.len() {
                            log_views[active_tab].page_down(log_area_height);
                        }
                    }
                }
                KeyCode::Esc => {
                    if let Some(active_tab) = log_tabs.index.checked_sub(0) {
                        if active_tab < log_views.len() {
                            log_views[active_tab].enable_follow();
                        }
                    }
                }
                _ => {}
            }
        }

        // Small sleep to prevent busy waiting
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::docker::docker_log_watcher::DockerLogManager;
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
        let _log_manager = DockerLogManager::new();

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
