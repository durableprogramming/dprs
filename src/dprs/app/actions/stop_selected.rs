// Implements container management functionality for stopping multiple selected containers.
// This module contains a function to stop multiple containers selected in visual mode
// by executing 'docker stop' commands for each selected container. After stopping containers,
// it automatically reloads the container list to reflect the current state.

use crate::dprs::app::state_machine::{AppState, ProgressUpdate};
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use crate::shared::config::Config;

pub fn stop_selected_containers(app_state: &mut AppState, config: &Config) -> Result<(), String> {
    let selected_indices = app_state.get_selected_indices();

    if selected_indices.is_empty() {
        return Err("No containers selected".to_string());
    }

    let displayed_containers = app_state.get_displayed_containers();
    let container_names: Vec<String> = selected_indices
        .iter()
        .filter_map(|&index| displayed_containers.get(index).map(|c| c.name.clone()))
        .collect();

    // Only show progress modal if experimental animation flag is set
    let tx = if config.general.experimental_fx {
        Some(app_state.start_progress(format!("Stopping {} containers...", container_names.len())))
    } else {
        None
    };

    thread::spawn(move || {
        let _ = stop_containers_async(container_names, tx);
    });

    Ok(())
}

fn stop_containers_async(container_names: Vec<String>, tx: Option<Sender<ProgressUpdate>>) -> Result<(), String> {
    let total = container_names.len();
    let mut stopped = 0;
    let mut errors = Vec::new();

    for (i, name) in container_names.into_iter().enumerate() {
        let progress = (i as f32 / total as f32) * 80.0 + 10.0;
        if let Some(ref sender) = tx {
            let _ = sender.send(ProgressUpdate::Update {
                message: format!("Stopping container {} ({}/{})...", name, i + 1, total),
                percentage: progress,
            });
        }

        thread::sleep(Duration::from_millis(50));

        match Command::new("docker")
            .arg("stop")
            .arg(&name)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    stopped += 1;
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    errors.push(format!("Failed to stop {}: {}", name, error));
                }
            }
            Err(e) => {
                errors.push(format!("Failed to execute docker stop for {}: {}", name, e));
            }
        }
    }

    if errors.is_empty() {
        if let Some(ref sender) = tx {
            let _ = sender.send(ProgressUpdate::Update {
                message: format!("Successfully stopped {} containers", stopped),
                percentage: 100.0,
            });
            let _ = sender.send(ProgressUpdate::Complete);
        }
        Ok(())
    } else {
        if let Some(ref sender) = tx {
            let _ = sender.send(ProgressUpdate::Error(format!("Some containers failed to stop: {}", errors.join(", "))));
        }
        Err(format!("Some containers failed to stop: {}", errors.join(", ")))
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.