//
//  Implements container management functionality for restarting Docker containers.
//  This module contains a function to restart the selected container by executing
//  a 'docker restart' command, allowing users to reset container execution directly
//  from the TUI interface. After restarting a container, it automatically reloads
//  the container list to reflect the current state.

pub use crate::dprs::app::state_machine::{AppState, ProgressUpdate};
use crate::shared::config::Config;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

pub fn restart_container(app_state: &mut AppState, config: &Config) -> Result<(), String> {
    let selected = app_state
        .list_state
        .selected()
        .ok_or("No container selected")?;

    let container = app_state
        .containers
        .get(selected)
        .ok_or("Invalid container index")?;

    let container_name = container.name.clone();

    // Only show progress modal if experimental animation flag is set
    let tx = if config.general.experimental_fx {
        Some(app_state.start_progress(format!("Restarting container {}...", container_name)))
    } else {
        None
    };

    thread::spawn(move || {
        let _ = restart_container_async(container_name, tx);
    });

    Ok(())
}

fn restart_container_async(
    container_name: String,
    tx: Option<Sender<ProgressUpdate>>,
) -> Result<(), String> {
    // Send initial progress if tx is present
    if let Some(ref sender) = tx {
        let _ = sender.send(ProgressUpdate::Update {
            message: format!("Restarting container {}...", container_name),
            percentage: 10.0,
        });
    }

    // Small delay to show progress
    thread::sleep(Duration::from_millis(100));

    // Execute docker restart
    let result = Command::new("docker")
        .arg("restart")
        .arg(&container_name)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                if let Some(ref sender) = tx {
                    let _ = sender.send(ProgressUpdate::Update {
                        message: format!("Container {} restarted successfully", container_name),
                        percentage: 100.0,
                    });
                    let _ = sender.send(ProgressUpdate::Complete);
                }
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                if let Some(ref sender) = tx {
                    let _ = sender.send(ProgressUpdate::Error(format!(
                        "Failed to restart {}: {}",
                        container_name, error
                    )));
                }
                Err(format!("Failed to restart container: {}", error))
            }
        }
        Err(e) => {
            if let Some(ref sender) = tx {
                let _ = sender.send(ProgressUpdate::Error(format!(
                    "Failed to execute docker restart: {}",
                    e
                )));
            }
            Err(format!("Failed to restart container: {}", e))
        }
    }
}

#[cfg(test)]
mod tests;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
