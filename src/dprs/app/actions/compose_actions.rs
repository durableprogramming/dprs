// Actions for Docker Compose project operations.
// These actions allow stopping, restarting, and managing entire compose projects at once.

use crate::dprs::app::state_machine::{AppState, ProgressUpdate};
use crate::dprs::display::compose_view::{group_containers_by_project, ComposeProject};
use crate::shared::config::Config;
use std::io::Error;
use std::process::Command;

pub fn restart_compose_project(
    app_state: &mut AppState,
    project_index: usize,
    config: &Config,
) -> Result<(), Error> {
    let projects = group_containers_by_project(app_state);

    if let Some(project) = projects.get(project_index) {
        restart_project(project, app_state, config)
    } else {
        Err(Error::other("Invalid project index"))
    }
}

pub fn stop_compose_project(
    app_state: &mut AppState,
    project_index: usize,
    config: &Config,
) -> Result<(), Error> {
    let projects = group_containers_by_project(app_state);

    if let Some(project) = projects.get(project_index) {
        stop_project(project, app_state, config)
    } else {
        Err(Error::other("Invalid project index"))
    }
}

pub fn restart_selected_compose_projects(
    app_state: &mut AppState,
    _config: &Config,
) -> Result<(), Error> {
    let projects = group_containers_by_project(app_state);
    let selected_indices = app_state.get_selected_indices();

    if selected_indices.is_empty() {
        return Err(Error::other("No projects selected"));
    }

    let total = selected_indices.len();
    let progress_sender = app_state.start_progress(format!("Restarting projects... (0/{})", total));

    std::thread::spawn(move || {
        for (idx, &project_idx) in selected_indices.iter().enumerate() {
            if let Some(project) = projects.get(project_idx) {
                let percentage = if total > 0 {
                    (idx as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                let _ = progress_sender.send(ProgressUpdate::Update {
                    message: format!(
                        "Restarting {} ({}/{})",
                        project.project_name,
                        idx + 1,
                        total
                    ),
                    percentage,
                });

                restart_project_sync(project);
            }
        }

        let _ = progress_sender.send(ProgressUpdate::Complete);
    });

    Ok(())
}

pub fn stop_selected_compose_projects(
    app_state: &mut AppState,
    _config: &Config,
) -> Result<(), Error> {
    let projects = group_containers_by_project(app_state);
    let selected_indices = app_state.get_selected_indices();

    if selected_indices.is_empty() {
        return Err(Error::other("No projects selected"));
    }

    let total = selected_indices.len();
    let progress_sender = app_state.start_progress(format!("Stopping projects... (0/{})", total));

    std::thread::spawn(move || {
        for (idx, &project_idx) in selected_indices.iter().enumerate() {
            if let Some(project) = projects.get(project_idx) {
                let percentage = if total > 0 {
                    (idx as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                let _ = progress_sender.send(ProgressUpdate::Update {
                    message: format!("Stopping {} ({}/{})", project.project_name, idx + 1, total),
                    percentage,
                });

                stop_project_sync(project);
            }
        }

        let _ = progress_sender.send(ProgressUpdate::Complete);
    });

    Ok(())
}

fn restart_project(
    project: &ComposeProject,
    app_state: &mut AppState,
    config: &Config,
) -> Result<(), Error> {
    let total = project.containers.len();
    let progress_sender = app_state.start_progress(format!(
        "Restarting project {} (0/{})",
        project.project_name, total
    ));

    let project_clone = project.clone();
    let show_progress = config.general.experimental_fx;

    std::thread::spawn(move || {
        for (idx, container_name) in project_clone.containers.iter().enumerate() {
            if show_progress {
                let percentage = if total > 0 {
                    (idx as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                let _ = progress_sender.send(ProgressUpdate::Update {
                    message: format!("Restarting {} ({}/{})", container_name, idx + 1, total),
                    percentage,
                });
            }

            let _ = Command::new("docker")
                .args(["restart", container_name])
                .output();
        }

        let _ = progress_sender.send(ProgressUpdate::Complete);
    });

    Ok(())
}

fn stop_project(
    project: &ComposeProject,
    app_state: &mut AppState,
    config: &Config,
) -> Result<(), Error> {
    let total = project.containers.len();
    let progress_sender = app_state.start_progress(format!(
        "Stopping project {} (0/{})",
        project.project_name, total
    ));

    let project_clone = project.clone();
    let show_progress = config.general.experimental_fx;

    std::thread::spawn(move || {
        for (idx, container_name) in project_clone.containers.iter().enumerate() {
            if show_progress {
                let percentage = if total > 0 {
                    (idx as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                let _ = progress_sender.send(ProgressUpdate::Update {
                    message: format!("Stopping {} ({}/{})", container_name, idx + 1, total),
                    percentage,
                });
            }

            let _ = Command::new("docker")
                .args(["stop", container_name])
                .output();
        }

        let _ = progress_sender.send(ProgressUpdate::Complete);
    });

    Ok(())
}

// Synchronous versions for use in threads
fn restart_project_sync(project: &ComposeProject) {
    for container_name in &project.containers {
        let _ = Command::new("docker")
            .args(["restart", container_name])
            .output();
    }
}

fn stop_project_sync(project: &ComposeProject) {
    for container_name in &project.containers {
        let _ = Command::new("docker")
            .args(["stop", container_name])
            .output();
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_percentage_calculation_with_zero_total() {
        // Test that percentage calculation doesn't panic with zero total
        let total = 0;
        let idx = 0;

        let percentage = if total > 0 {
            (idx as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        assert_eq!(percentage, 0.0);
    }

    #[test]
    fn test_percentage_calculation_with_normal_values() {
        let total = 4;

        // Test first iteration
        let idx = 0;
        let percentage = if total > 0 {
            (idx as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        assert_eq!(percentage, 0.0);

        // Test middle iteration
        let idx = 2;
        let percentage = if total > 0 {
            (idx as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        assert_eq!(percentage, 50.0);

        // Test last iteration
        let idx = 3;
        let percentage = if total > 0 {
            (idx as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        assert_eq!(percentage, 75.0);
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
