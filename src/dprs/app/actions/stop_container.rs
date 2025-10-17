//
//  Implements container management functionality for stopping Docker containers.
//  This module contains a function to stop the selected container by executing
//  a 'docker stop' command, allowing users to halt container execution directly
//  from the TUI interface. After stopping a container, it automatically reloads
//  the container list to reflect the current state.

use crate::dprs::app::state_machine::AppState;
use std::process::Command;

pub fn stop_container(app_state: &mut AppState) -> Result<(), String> {
    let selected = app_state
        .list_state
        .selected()
        .ok_or("No container selected")?;

    let container = app_state
        .containers
        .get(selected)
        .ok_or("Invalid container index")?;

    Command::new("docker")
        .arg("stop")
        .arg(&container.name)
        .output()
        .map_err(|e| format!("Failed to stop container: {}", e))?;

    // Reload containers to reflect the changes
    app_state.load_containers();

    Ok(())
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
