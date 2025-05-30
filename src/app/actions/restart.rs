// 
//  Implements container management functionality for restarting Docker containers.
//  This module contains a function to restart the selected container by executing
//  a 'docker restart' command, allowing users to reset container execution directly
//  from the TUI interface. After restarting a container, it automatically reloads
//  the container list to reflect the current state.

pub use crate::app::state_machine::AppState;
use std::process::Command;

pub fn restart_container(app_state: &mut AppState) -> Result<(), String> {
    let selected = app_state
        .list_state
        .selected()
        .ok_or("No container selected")?;

    let container = app_state
        .containers
        .get(selected)
        .ok_or("Invalid container index")?;

    Command::new("docker")
        .args(["restart", &container.name])
        .output()
        .map_err(|e| format!("Failed to restart container: {}", e))?;

    // Reload containers to reflect the changes
    app_state.load_containers();

    Ok(())
}

#[cfg(test)]
mod tests;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
