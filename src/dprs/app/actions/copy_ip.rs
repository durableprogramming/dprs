//  Implements clipboard functionality for Docker container IP addresses.
//  This module contains a function to copy the selected container's IP address
//  to the system clipboard, allowing users to easily use container IPs in other applications.

use copypasta_ext::prelude::*;
use copypasta_ext::x11_bin::ClipboardContext;

use crate::dprs::app::state_machine::AppState;

pub fn copy_ip_address(app_state: &AppState) -> Result<String, String> {
    let selected = app_state
        .list_state
        .selected()
        .ok_or("No container selected")?;

    let container = app_state
        .containers
        .get(selected)
        .ok_or("Invalid container index")?;

    // Extract only the first IP address
    // IP addresses can be concatenated without separators when multiple networks exist
    let first_ip = extract_first_ip(&container.ip_address);

    let mut ctx: ClipboardContext = ClipboardContext::new().unwrap();

    ctx.set_contents(first_ip.to_owned())
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    Ok(first_ip)
}

/// Extracts the first IP address from a string that may contain multiple IPs
/// IP addresses may be separated by whitespace, commas, or concatenated directly
fn extract_first_ip(ip_string: &str) -> String {
    // Trim any leading/trailing whitespace first
    let trimmed = ip_string.trim();

    // First try splitting by comma or whitespace
    if let Some(first) = trimmed.split(&[',', ' '][..]).next() {
        let first_trimmed = first.trim();
        if !first_trimmed.is_empty() {
            return first_trimmed.to_string();
        }
    }

    // If no separators, parse character by character to extract first valid IP
    let mut result = String::new();
    let mut dot_count = 0;

    for ch in trimmed.chars() {
        if ch.is_ascii_digit() || ch == '.' || ch == ':' {
            result.push(ch);
            if ch == '.' {
                dot_count += 1;
            }
            // IPv4 has 3 dots, stop after we've captured a complete IP
            if dot_count == 3 {
                // Continue until we hit a non-IP character or complete the last octet
                for next_ch in trimmed[result.len()..].chars() {
                    if next_ch.is_ascii_digit() {
                        result.push(next_ch);
                    } else {
                        break;
                    }
                }
                break;
            }
        } else if !result.is_empty() {
            // Hit a non-IP character after starting to collect
            break;
        }
    }

    if result.is_empty() {
        trimmed.to_string()
    } else {
        result
    }
}

#[cfg(test)]
mod tests;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
