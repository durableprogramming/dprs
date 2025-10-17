// The dplw (Docker Process Log Watcher) binary provides a terminal user
// interface for monitoring logs from Docker containers in real-time. It
// allows users to view logs from multiple containers simultaneously,
// switch between containers with arrow keys, scroll through logs, and
// refresh the container list. This file contains the main application loop
// and UI rendering logic.

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, io::stdout};

use dprs::dplw::main_loop::run_app;
use dprs::shared::docker::docker_log_watcher::DockerLogManager;

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut log_manager = DockerLogManager::new();
    log_manager.start_watching_all_containers()?;

    // Ensure cleanup happens even if there's a panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_app(&mut terminal, &mut log_manager)
    }));

    // Always restore terminal, regardless of what happened
    let _ = disable_raw_mode();
    let _ = stdout().execute(LeaveAlternateScreen);

    // Always stop log manager
    log_manager.stop_all();

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => {
            println!("Error: {}", err);
            Err(err)
        }
        Err(_) => {
            println!("Application panicked");
            std::process::exit(1);
        }
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
