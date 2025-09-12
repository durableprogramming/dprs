// The docker_log_watcher module implements real-time Docker container log monitoring.
// It provides two main components:
// - DockerLogWatcher: handles log collection for a single container using bollard
// - DockerLogManager: coordinates multiple watchers and provides container discovery
// The module supports starting/stopping log collection, retrieving collected logs,
// and refreshing the container list. It ensures proper resource cleanup with async
// tasks and implements graceful shutdown through Drop trait implementation.

use bollard::container::{ListContainersOptions, LogsOptions};
use bollard::models::ContainerSummary;
use bollard::Docker;
use std::collections::{HashMap, VecDeque};
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct DockerLogWatcher {
    container_name: String,
    logs: Arc<Mutex<VecDeque<String>>>,
    max_logs: usize,
    handle: Option<JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
}

impl DockerLogWatcher {
    pub fn new(container_name: String, max_logs: usize) -> Self {
        Self {
            container_name,
            logs: Arc::new(Mutex::new(VecDeque::with_capacity(max_logs))),
            max_logs,
            handle: None,
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        let container_name = self.container_name.clone();
        let logs = Arc::clone(&self.logs);
        let max_logs = self.max_logs;
        let running = Arc::clone(&self.running);

        // Set running state to true
        *running.lock().unwrap() = true;

        let handle = thread::spawn(move || {
            // Create a tokio runtime for this thread
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("Failed to create tokio runtime: {}", e);
                    return;
                }
            };

            rt.block_on(async {
                let docker = match Docker::connect_with_defaults() {
                    Ok(docker) => docker,
                    Err(e) => {
                        eprintln!("Failed to connect to Docker: {}", e);
                        return;
                    }
                };

                let options = Some(LogsOptions::<String> {
                    follow: true,
                    stdout: true,
                    stderr: true,
                    tail: "100".to_string(),
                    ..Default::default()
                });

                let mut stream = docker.logs(&container_name, options);

                use futures_util::stream::StreamExt;
                while let Some(log_result) = stream.next().await {
                    if !*running.lock().unwrap() {
                        break;
                    }

                    match log_result {
                        Ok(log_line) => {
                            let log_str = match log_line {
                                bollard::container::LogOutput::StdOut { message } => {
                                    String::from_utf8_lossy(&message).trim_end().to_string()
                                }
                                bollard::container::LogOutput::StdErr { message } => {
                                    format!("ERROR: {}", String::from_utf8_lossy(&message).trim_end())
                                }
                                bollard::container::LogOutput::Console { message } => {
                                    String::from_utf8_lossy(&message).trim_end().to_string()
                                }
                                bollard::container::LogOutput::StdIn { message: _ } => continue,
                            };

                            let mut logs = logs.lock().unwrap();
                            logs.push_back(log_str);
                            while logs.len() > max_logs {
                                logs.pop_front();
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading logs for {}: {}", container_name, e);
                            break;
                        }
                    }
                }
            });
        });

        self.handle = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            // Set running state to false
            *self.running.lock().unwrap() = false;

            // Wait for the thread to finish
            let _ = handle.join();
        }
    }

    pub fn get_logs(&self) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        logs.iter().cloned().collect()
    }

    pub fn container_name(&self) -> &str {
        &self.container_name
    }
}

pub struct DockerLogManager {
    watchers: Vec<DockerLogWatcher>,
}

impl Default for DockerLogManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerLogManager {
    pub fn new() -> Self {
        Self {
            watchers: Vec::new(),
        }
    }

    pub fn start_watching_container(&mut self, container_name: String) -> Result<(), Error> {
        let mut watcher = DockerLogWatcher::new(container_name, 1000);
        watcher.start()?;
        self.watchers.push(watcher);
        Ok(())
    }

    pub fn start_watching_all_containers(&mut self) -> Result<(), Error> {
        // Create a tokio runtime for this synchronous function
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to create runtime: {}", e)))?;

        let container_names = rt.block_on(async {
            let docker = Docker::connect_with_defaults()
                .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to connect to Docker: {}", e)))?;

            let options = Some(ListContainersOptions::<String> {
                all: false,
                ..Default::default()
            });

            let containers = docker
                .list_containers(options)
                .await
                .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to list containers: {}", e)))?;

            let names: Vec<String> = containers
                .into_iter()
                .filter_map(|container| {
                    container.names.and_then(|names| {
                        names.first().map(|name| {
                            // Remove leading slash from container name
                            name.strip_prefix('/').unwrap_or(name).to_string()
                        })
                    })
                })
                .collect();

            Ok::<Vec<String>, Error>(names)
        })?;

        // Clear existing watchers
        self.stop_all();
        self.watchers.clear();

        // Start watching each container
        for container_name in container_names {
            if !container_name.is_empty() {
                self.start_watching_container(container_name)?;
            }
        }

        Ok(())
    }

    pub fn stop_all(&mut self) {
        for watcher in &mut self.watchers {
            watcher.stop();
        }
    }

    pub fn get_watcher(&self, index: usize) -> Option<&DockerLogWatcher> {
        self.watchers.get(index)
    }

    pub fn watcher_count(&self) -> usize {
        self.watchers.len()
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        self.start_watching_all_containers()
    }
}

impl Drop for DockerLogManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_docker_log_watcher_new() {
        let watcher = DockerLogWatcher::new("test-container".to_string(), 100);
        assert_eq!(watcher.container_name(), "test-container");
        assert_eq!(watcher.get_logs().len(), 0);
    }

    #[test]
    fn test_docker_log_manager_new() {
        let manager = DockerLogManager::new();
        assert_eq!(manager.watcher_count(), 0);
    }

    #[test]
    fn test_docker_log_manager_default() {
        let manager = DockerLogManager::default();
        assert_eq!(manager.watcher_count(), 0);
    }

}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.