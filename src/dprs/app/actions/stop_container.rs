//
//  Implements container management functionality for stopping Docker containers.
//  This module contains a function to stop the selected container by executing
//  a 'docker stop' command, allowing users to halt container execution directly
//  from the TUI interface. The command runs on a background thread so the UI
//  stays responsive: `docker stop` sends SIGTERM and then waits out a grace
//  period (10 seconds by default) before killing the container, which would
//  otherwise freeze the event loop for the duration.
//
//  The container list is refreshed by the main loop's periodic reload rather
//  than here, since this function returns before the container has stopped.

use crate::dprs::app::state_machine::{AppState, ProgressUpdate};
use crate::shared::config::Config;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;

pub fn stop_container(app_state: &mut AppState, config: &Config) -> Result<(), String> {
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
        Some(app_state.start_progress(format!("Stopping container {}...", container_name)))
    } else {
        None
    };

    thread::spawn(move || {
        let _ = stop_container_async(container_name, tx);
    });

    Ok(())
}

fn stop_container_async(
    container_name: String,
    tx: Option<Sender<ProgressUpdate>>,
) -> Result<(), String> {
    if let Some(ref sender) = tx {
        let _ = sender.send(ProgressUpdate::Update {
            message: format!("Stopping container {}...", container_name),
            percentage: 10.0,
        });
    }

    let result = Command::new("docker")
        .arg("stop")
        .arg(&container_name)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                if let Some(ref sender) = tx {
                    let _ = sender.send(ProgressUpdate::Update {
                        message: format!("Container {} stopped successfully", container_name),
                        percentage: 100.0,
                    });
                    let _ = sender.send(ProgressUpdate::Complete);
                }
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                if let Some(ref sender) = tx {
                    let _ = sender.send(ProgressUpdate::Error(format!(
                        "Failed to stop {}: {}",
                        container_name, error
                    )));
                }
                Err(format!("Failed to stop container: {}", error))
            }
        }
        Err(e) => {
            if let Some(ref sender) = tx {
                let _ = sender.send(ProgressUpdate::Error(format!(
                    "Failed to execute docker stop: {}",
                    e
                )));
            }
            Err(format!("Failed to stop container: {}", e))
        }
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
