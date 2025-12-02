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

    // Parse character by character to extract first valid IP
    let mut result = String::new();
    let mut dot_count = 0;
    let mut char_indices = trimmed.char_indices().peekable();

    while let Some((_idx, ch)) = char_indices.next() {
        if ch.is_ascii_digit() {
            result.push(ch);
        } else if ch == '.' {
            result.push(ch);
            dot_count += 1;
            // IPv4 has 3 dots, stop after we've captured a complete IP
            if dot_count == 3 {
                // Continue until we hit a non-digit to complete the last octet
                // IPv4 octets are at most 3 digits
                let mut octet_digits = 0;
                while let Some((_, next_ch)) = char_indices.peek() {
                    if next_ch.is_ascii_digit() && octet_digits < 3 {
                        result.push(*next_ch);
                        char_indices.next();
                        octet_digits += 1;
                    } else {
                        break;
                    }
                }
                break;
            }
        } else if ch == ',' || ch.is_whitespace() {
            // If we've collected an IP, stop here
            if !result.is_empty() {
                break;
            }
            // Otherwise skip the separator and continue
        } else {
            // Hit a non-IP character after starting to collect
            if !result.is_empty() {
                break;
            }
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
