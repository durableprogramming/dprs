// /**
// * Implements clipboard functionality for Docker container IP addresses.
// * This module contains a function to copy the selected container's IP address
// * to the system clipboard, allowing users to easily use container IPs in other applications.
// */
use clipboard::{ClipboardContext, ClipboardProvider};

use crate::app::state_machine::AppState;

pub fn copy_ip_address(app_state: &AppState) -> Result<(), String> {
    let selected = app_state
        .list_state
        .selected()
        .ok_or("No container selected")?;

    let container = app_state
        .containers
        .get(selected)
        .ok_or("Invalid container index")?;

    let mut ctx: ClipboardContext =
        ClipboardProvider::new().map_err(|e| format!("Failed to initialize clipboard: {}", e))?;

    ctx.set_contents(container.ip_address.clone())
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    Ok(())
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
