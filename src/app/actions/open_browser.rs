// /**
// * Implements browser functionality to open Docker container web interfaces.
// * This module contains a function to open the selected container's IP address
// * in the default system browser using platform-specific commands, allowing
// * users to quickly access container web interfaces.
// */
use crate::app::state_machine::AppState;
use std::process::Command;

pub fn open_browser(app_state: &AppState) -> Result<(), String> {
    let selected = app_state
        .list_state
        .selected()
        .ok_or("No container selected")?;

    let container = app_state
        .containers
        .get(selected)
        .ok_or("Invalid container index")?;

    let url = format!("http://{}", container.ip_address);

    #[cfg(target_os = "linux")]
    Command::new("xdg-open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;

    #[cfg(target_os = "macos")]
    Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;

    #[cfg(target_os = "windows")]
    Command::new("cmd")
        .args(["/c", "start", &url])
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;

    Ok(())
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
