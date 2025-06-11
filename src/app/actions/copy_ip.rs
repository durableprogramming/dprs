//  Implements clipboard functionality for Docker container IP addresses.
//  This module contains a function to copy the selected container's IP address
//  to the system clipboard, allowing users to easily use container IPs in other applications.

use copypasta_ext::prelude::*;
use copypasta_ext::x11_bin::ClipboardContext;

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

    let mut ctx: ClipboardContext = ClipboardContext::new().unwrap();

    ctx.set_contents(container.ip_address.clone().to_owned()).map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
