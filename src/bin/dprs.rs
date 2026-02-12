// The dprs (Docker Process Manager) binary provides a terminal user interface
// for managing Docker containers. It allows users to list running containers,
// view details, stop containers, copy IP addresses, and open web interfaces.
// This file contains the main application loop and UI rendering logic for dprs.

use crossterm::{
    event::Event,
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

use dprs::dprs::app::{actions, AppState};
use dprs::dprs::commands::{CommandExecutor, CommandResult};
use dprs::dprs::display;
use dprs::dprs::display::toast::ToastManager;
use dprs::dprs::modes::Mode;
use dprs::shared::config::Config;
use dprs::shared::input::input_watcher::InputWatcher;
use tachyonfx::EffectManager;

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // Load configuration and create app state
    let config = Config::load();
    let mut toast_manager = ToastManager::new();

    let result = run_app(&mut terminal, &mut toast_manager, config);

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
    mut config: Config,
) -> Result<(), io::Error> {
    let mut last_refresh = Instant::now();
    let refresh_interval = if config.should_auto_refresh() {
        config.auto_refresh_interval()
    } else {
        Duration::from_millis(500) // Default refresh interval
    };
    let mut app_state = AppState::new();
    let mut command_executor = CommandExecutor::new();
    let input_watcher = InputWatcher::new();
    let mut effects: EffectManager<()> = EffectManager::default();
    let mut last_frame = Instant::now();

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
        // Calculate elapsed time for effects
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();

        // Update progress
        app_state.update_progress();

        // Draw UI
        terminal.draw(|f| {
            display::draw::<B>(
                f,
                &mut app_state,
                toast_manager,
                &mut config,
                &mut effects,
                elapsed,
            )
        })?;

        // Handle toast expiration
        toast_manager.check_expired();

        // Handle input events from watcher
        if let Ok(Event::Key(key)) = input_watcher.try_recv() {
            handle_key_event(
                key,
                &mut app_state,
                &mut command_executor,
                toast_manager,
                &mut config,
            );
        }

        // Small sleep to prevent busy waiting
        std::thread::sleep(Duration::from_millis(10));

        // Periodic refresh check (if poll timed out or no event handled that resets the timer)
        if last_refresh.elapsed() >= refresh_interval {
            if let Err(e) = app_state.refresh_containers() {
                toast_manager.show(&format!("Auto-refresh error: {}", e), 3000);
            }
            // No toast for successful auto-refresh to avoid being too noisy.
            last_refresh = Instant::now();
        }

        // Check for exit request
        if app_state.should_exit() {
            return Ok(());
        }
    }
}

fn handle_key_event(
    key: crossterm::event::KeyEvent,
    app_state: &mut AppState,
    command_executor: &mut CommandExecutor,
    toast_manager: &mut ToastManager,
    config: &mut Config,
) {
    match app_state.mode {
        Mode::Normal => handle_normal_mode(key, app_state, toast_manager, config),
        Mode::Visual => handle_visual_mode(key, app_state, toast_manager, config),
        Mode::Command => {
            handle_command_mode(key, app_state, command_executor, toast_manager, config)
        }
        Mode::Search => handle_search_mode(key, app_state, toast_manager),
    }
}

fn handle_normal_mode(
    key: crossterm::event::KeyEvent,
    app_state: &mut AppState,
    toast_manager: &mut ToastManager,
    config: &mut Config,
) {
    use crossterm::event::{KeyCode, KeyModifiers};

    // Handle context menu if active
    if app_state.context_menu.active {
        handle_context_menu_keys(key, app_state, toast_manager, config);
        return;
    }

    match key.code {
        // Context menu
        KeyCode::Char('.') => {
            let container = app_state.get_selected_container().cloned();
            let project = if app_state.compose_view_mode {
                use dprs::dprs::display::compose_view::group_containers_by_project;
                let projects = group_containers_by_project(app_state);
                app_state
                    .list_state
                    .selected()
                    .and_then(|idx| projects.get(idx).cloned())
            } else {
                None
            };
            app_state.context_menu.activate(container, project, config);
        }

        // Quit
        KeyCode::Char('q') => app_state.request_exit(),

        // Basic navigation
        KeyCode::Char('j') | KeyCode::Down => app_state.next(),
        KeyCode::Char('k') | KeyCode::Up => app_state.previous(),

        // Vim-style navigation
        KeyCode::Char('g') => {
            // Handle gg sequence - for simplicity, just go to first for now
            app_state.go_to_first();
        }
        KeyCode::Char('G') => app_state.go_to_last(),
        KeyCode::Char('w') => app_state.word_next(),
        KeyCode::Char('b') => app_state.word_previous(),
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app_state.half_page_up()
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app_state.half_page_down()
        }

        // Mode switching
        KeyCode::Char('v') => app_state.enter_visual_mode(),
        KeyCode::Char(':') => app_state.enter_command_mode(),
        KeyCode::Char('/') => app_state.enter_search_mode(true),
        KeyCode::Char('?') => app_state.enter_search_mode(false),

        // Search navigation
        KeyCode::Char('n') => app_state.next_search_result(),
        KeyCode::Char('N') => app_state.previous_search_result(),

        // Container/Project actions (behavior depends on compose_view_mode)
        KeyCode::Char('s') => {
            if app_state.compose_view_mode {
                if let Some(selected) = app_state.list_state.selected() {
                    match actions::stop_compose_project(app_state, selected, &*config) {
                        Ok(_) => {
                            toast_manager.show("Stop command sent for project. Refreshing...", 2000)
                        }
                        Err(e) => {
                            toast_manager.show(&format!("Error stopping project: {}", e), 3000)
                        }
                    }
                }
            } else {
                match actions::stop_container(app_state) {
                    Ok(_) => toast_manager.show("Stop command sent. Refreshing list...", 2000),
                    Err(e) => toast_manager.show(&format!("Error stopping container: {}", e), 3000),
                }
            }
        }
        KeyCode::Char('c') => {
            if !app_state.compose_view_mode {
                match actions::copy_ip_address(app_state) {
                    Ok(ip) => {
                        toast_manager.show(&format!("IP address copied to clipboard: {}", ip), 2000)
                    }
                    Err(e) => toast_manager.show(&format!("Error copying IP: {}", e), 3000),
                }
            }
        }
        KeyCode::Char('o') => {
            if !app_state.compose_view_mode {
                match actions::open_browser(app_state) {
                    Ok(_) => toast_manager.show("Opening browser...", 2000),
                    Err(e) => toast_manager.show(&format!("Error opening browser: {}", e), 3000),
                }
            }
        }
        KeyCode::Char('r') => {
            if app_state.compose_view_mode {
                if let Some(selected) = app_state.list_state.selected() {
                    match actions::restart_compose_project(app_state, selected, &*config) {
                        Ok(_) => toast_manager
                            .show("Restart command sent for project. Refreshing...", 2000),
                        Err(e) => {
                            toast_manager.show(&format!("Error restarting project: {}", e), 3000)
                        }
                    }
                }
            } else {
                match actions::restart_container(app_state, &*config) {
                    Ok(_) => toast_manager.show("Restart command sent. Refreshing list...", 2000),
                    Err(e) => {
                        toast_manager.show(&format!("Error restarting container: {}", e), 3000)
                    }
                }
            }
        }
        KeyCode::Char('t') => {
            app_state.tabular_mode = !app_state.tabular_mode;
            let mode_text = if app_state.tabular_mode {
                "tabular"
            } else {
                "normal"
            };
            toast_manager.show(&format!("Switched to {} view", mode_text), 1500);
        }
        KeyCode::Char('p') => {
            app_state.compose_view_mode = !app_state.compose_view_mode;
            let mode_text = if app_state.compose_view_mode {
                "compose projects"
            } else {
                "containers"
            };
            // Reset selection when toggling view
            app_state.list_state.select(Some(0));
            app_state.table_state.select(Some(0));
            toast_manager.show(&format!("Switched to {} view", mode_text), 1500);
        }

        // Filter
        KeyCode::Char('f') => app_state.enter_filter_mode(),

        // Container filter toggles
        KeyCode::Char('+') => {
            app_state.toggle_recent();
            let filter_name = app_state.container_filter.display_name();
            toast_manager.show(&format!("Switched to {} containers", filter_name), 1500);
        }
        KeyCode::Char('!') => {
            app_state.toggle_all();
            let filter_name = app_state.container_filter.display_name();
            toast_manager.show(&format!("Switched to {} containers", filter_name), 1500);
        }

        KeyCode::Esc => {
            if !app_state.filter_text.is_empty() {
                app_state.clear_filter();
                toast_manager.show("Filter cleared", 1500);
            } else if !app_state.search_state.matches.is_empty() {
                app_state.search_state.clear();
                toast_manager.show("Search cleared", 1500);
            }
        }

        // Legacy support for old filter mode
        _ => {
            if app_state.filter_mode {
                handle_filter_input(key, app_state);
            }
        }
    }
}

fn handle_visual_mode(
    key: crossterm::event::KeyEvent,
    app_state: &mut AppState,
    toast_manager: &mut ToastManager,
    config: &mut Config,
) {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app_state.next();
            if let Some(ref mut selection) = app_state.visual_selection {
                if let Some(current) = app_state.list_state.selected() {
                    selection.extend_to(current);
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app_state.previous();
            if let Some(ref mut selection) = app_state.visual_selection {
                if let Some(current) = app_state.list_state.selected() {
                    selection.extend_to(current);
                }
            }
        }
        KeyCode::Char('G') => {
            app_state.go_to_last();
        }
        KeyCode::Char('g') => {
            app_state.go_to_first();
        }
        KeyCode::Char('s') => {
            if app_state.compose_view_mode {
                match actions::stop_selected_compose_projects(app_state, &*config) {
                    Ok(_) => {
                        let count = app_state.get_selected_indices().len();
                        toast_manager.show(
                            &format!(
                                "Stopped {} project{}",
                                count,
                                if count == 1 { "" } else { "s" }
                            ),
                            2000,
                        );
                    }
                    Err(e) => toast_manager.show(&format!("Error stopping projects: {}", e), 3000),
                }
            } else {
                match actions::stop_selected_containers(app_state, &*config) {
                    Ok(_) => {
                        let count = app_state.get_selected_indices().len();
                        toast_manager.show(
                            &format!(
                                "Stopped {} container{}",
                                count,
                                if count == 1 { "" } else { "s" }
                            ),
                            2000,
                        );
                    }
                    Err(e) => {
                        toast_manager.show(&format!("Error stopping containers: {}", e), 3000)
                    }
                }
            }
            app_state.enter_normal_mode();
        }
        KeyCode::Char('r') => {
            if app_state.compose_view_mode {
                match actions::restart_selected_compose_projects(app_state, &*config) {
                    Ok(_) => {
                        let count = app_state.get_selected_indices().len();
                        toast_manager.show(
                            &format!(
                                "Restarted {} project{}",
                                count,
                                if count == 1 { "" } else { "s" }
                            ),
                            2000,
                        );
                    }
                    Err(e) => {
                        toast_manager.show(&format!("Error restarting projects: {}", e), 3000)
                    }
                }
            } else {
                match actions::restart_selected_containers(app_state, &*config) {
                    Ok(_) => {
                        let count = app_state.get_selected_indices().len();
                        toast_manager.show(
                            &format!(
                                "Restarted {} container{}",
                                count,
                                if count == 1 { "" } else { "s" }
                            ),
                            2000,
                        );
                    }
                    Err(e) => {
                        toast_manager.show(&format!("Error restarting containers: {}", e), 3000)
                    }
                }
            }
            app_state.enter_normal_mode();
        }
        KeyCode::Esc => app_state.enter_normal_mode(),
        _ => {}
    }
}

fn handle_command_mode(
    key: crossterm::event::KeyEvent,
    app_state: &mut AppState,
    command_executor: &mut CommandExecutor,
    toast_manager: &mut ToastManager,
    config: &mut Config,
) {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Enter => {
            let command = app_state.command_state.input.clone();
            match command_executor.execute(&command, app_state) {
                CommandResult::Success(msg) => {
                    toast_manager.show(&msg, 2000);
                    app_state.command_state.add_to_history(command);
                }
                CommandResult::Error(msg) => {
                    toast_manager.show(&format!("Error: {}", msg), 3000);
                }
                CommandResult::Navigation(line) => {
                    app_state.list_state.select(Some(line));
                    app_state.table_state.select(Some(line));
                    toast_manager.show(&format!("Jumped to line {}", line + 1), 1500);
                }
                CommandResult::Quit => {
                    app_state.request_exit();
                }
                CommandResult::ConfigReload(new_config) => {
                    *config = *new_config;
                    toast_manager.show("Configuration reloaded", 2000);
                    app_state.command_state.add_to_history(command);
                }
            }
            app_state.enter_normal_mode();
        }
        KeyCode::Esc => {
            app_state.enter_normal_mode();
        }
        _ => {
            app_state.command_state.handle_key(key);
        }
    }
}

fn handle_search_mode(
    key: crossterm::event::KeyEvent,
    app_state: &mut AppState,
    toast_manager: &mut ToastManager,
) {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Enter => {
            let query = app_state.search_state.query.clone();
            app_state.perform_search(&query);

            let matches_count = app_state.search_state.matches.len();
            if matches_count > 0 {
                app_state.next_search_result();
                toast_manager.show(&format!("Found {} matches", matches_count), 2000);
            } else {
                toast_manager.show("No matches found", 2000);
            }
            app_state.enter_normal_mode();
        }
        KeyCode::Esc => {
            app_state.enter_normal_mode();
        }
        KeyCode::Char(c) => {
            app_state.search_state.query.push(c);
            // Perform incremental search
            let query = app_state.search_state.query.clone();
            app_state.perform_search(&query);
        }
        KeyCode::Backspace => {
            app_state.search_state.query.pop();
            let query = app_state.search_state.query.clone();
            if !query.is_empty() {
                app_state.perform_search(&query);
            } else {
                app_state.search_state.clear();
            }
        }
        _ => {}
    }
}

fn handle_filter_input(key: crossterm::event::KeyEvent, app_state: &mut AppState) {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Enter => app_state.exit_filter_mode(),
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
}

fn handle_context_menu_keys(
    key: crossterm::event::KeyEvent,
    app_state: &mut AppState,
    toast_manager: &mut ToastManager,
    _config: &Config,
) {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app_state.context_menu.next();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app_state.context_menu.previous();
        }
        KeyCode::Enter => {
            if let Some(command) = app_state.context_menu.execute_selected_action() {
                // Execute the command in a shell
                toast_manager.show("Executing action...", 2000);

                std::thread::spawn(move || {
                    use std::process::Command;
                    let _ = Command::new("sh").arg("-c").arg(&command).spawn();
                });

                app_state.context_menu.deactivate();
            }
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('.') => {
            app_state.context_menu.deactivate();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn basic_startup_test() {
        let result = std::panic::catch_unwind(|| {
            let _app_state = dprs::dprs::app::AppState::new();
            let _toast_manager = dprs::dprs::display::toast::ToastManager::new();
        });
        assert!(result.is_ok());
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
