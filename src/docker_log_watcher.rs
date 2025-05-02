// The docker_log_watcher module implements real-time Docker container log monitoring.
// It provides two main components:
// - DockerLogWatcher: handles log collection for a single container by spawning
// background threads that execute "docker logs" commands and capture output
// - DockerLogManager: coordinates multiple watchers and provides container discovery
// The module supports starting/stopping log collection, retrieving collected logs,
// and refreshing the container list. It ensures proper resource cleanup with thread
// management and implements graceful shutdown through Drop trait implementation.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};
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
            let mut cmd = Command::new("docker")
                .args(["logs", "--follow", "--tail", "100", &container_name])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start docker logs command");

            let stdout = cmd.stdout.take().expect("Failed to get stdout");
            let stderr = cmd.stderr.take().expect("Failed to get stderr");

            // Combine stdout and stderr into a single reader
            let stdout_reader = BufReader::new(stdout);
            let stderr_reader = BufReader::new(stderr);

            // Handle stdout in a separate thread
            let logs_clone = Arc::clone(&logs);
            let running_clone = Arc::clone(&running);
            let stdout_handle = thread::spawn(move || {
                for line in stdout_reader.lines() {
                    if !*running_clone.lock().unwrap() {
                        break;
                    }

                    if let Ok(line) = line {
                        let mut logs = logs_clone.lock().unwrap();
                        logs.push_back(line);
                        while logs.len() > max_logs {
                            logs.pop_front();
                        }
                    }
                }
            });

            // Handle stderr in a separate thread
            let logs_clone = Arc::clone(&logs);
            let running_clone = Arc::clone(&running);
            let stderr_handle = thread::spawn(move || {
                for line in stderr_reader.lines() {
                    if !*running_clone.lock().unwrap() {
                        break;
                    }

                    if let Ok(line) = line {
                        let mut logs = logs_clone.lock().unwrap();
                        logs.push_back(format!("ERROR: {}", line));
                        while logs.len() > max_logs {
                            logs.pop_front();
                        }
                    }
                }
            });

            let running_clone = Arc::clone(&running);
            let _watcher = thread::spawn(move || loop {
                if !*running_clone.lock().unwrap() {
                    let _ = cmd.kill();
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            });
            // Wait for both readers to complete
            stdout_handle.join().unwrap();
            stderr_handle.join().unwrap();
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
        // Get list of running containers
        let output = Command::new("docker")
            .args(["ps", "--format", "{{.Names}}"])
            .output()
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to execute docker command: {}", e),
                )
            })?;

        if !output.status.success() {
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Docker command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Clear existing watchers
        self.stop_all();
        self.watchers.clear();

        // Start watching each container
        for line in output_str.lines() {
            let container_name = line.trim().to_string();
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
    use std::time::Duration;
    use std::thread;

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

    #[cfg(feature = "integration_tests")]
    mod integration {
        use super::*;

        fn setup_test_container() -> Result<String, std::io::Error> {
            let container_name = format!("test-container-{}", std::process::id());
            Command::new("docker")
                .args(["run", "-d", "--name", &container_name, "alpine", "sh", "-c", "while true; do echo hello world; sleep 1; done"])
                .output()?;
            
            Ok(container_name)
        }

        fn cleanup_test_container(container_name: &str) {
            let _ = Command::new("docker")
                .args(["rm", "-f", container_name])
                .output();
        }

        #[test]
        fn test_docker_log_watcher_start_stop() -> Result<(), std::io::Error> {
            let container_name = setup_test_container()?;
            
            let mut watcher = DockerLogWatcher::new(container_name.clone(), 100);
            watcher.start()?;
            
            // Wait for some logs to appear
            thread::sleep(Duration::from_secs(2));
            
            let logs = watcher.get_logs();
            assert!(!logs.is_empty(), "Should have collected some logs");
            assert!(logs.iter().any(|log| log.contains("hello world")));
            
            watcher.stop();
            
            let log_count = watcher.get_logs().len();
            thread::sleep(Duration::from_secs(2));
            
            // Log count should remain the same after stopping
            assert_eq!(watcher.get_logs().len(), log_count);
            
            cleanup_test_container(&container_name);
            Ok(())
        }

        #[test]
        fn test_docker_log_manager_start_watching_container() -> Result<(), std::io::Error> {
            let container_name = setup_test_container()?;
            
            let mut manager = DockerLogManager::new();
            manager.start_watching_container(container_name.clone())?;
            
            assert_eq!(manager.watcher_count(), 1);
            
            // Wait for logs to appear
            thread::sleep(Duration::from_secs(2));
            
            let watcher = manager.get_watcher(0).unwrap();
            let logs = watcher.get_logs();
            assert!(!logs.is_empty(), "Should have collected some logs");
            
            cleanup_test_container(&container_name);
            Ok(())
        }

        #[test]
        fn test_docker_log_manager_start_watching_all() -> Result<(), std::io::Error> {
            let container_name1 = setup_test_container()?;
            let container_name2 = {
                let name = format!("test-container-2-{}", std::process::id());
                Command::new("docker")
                    .args(["run", "-d", "--name", &name, "alpine", "sh", "-c", "while true; do echo container 2; sleep 1; done"])
                    .output()?;
                name
            };
            
            let mut manager = DockerLogManager::new();
            manager.start_watching_all_containers()?;
            
            assert!(manager.watcher_count() >= 2, "Should be watching at least our two test containers");
            
            // Wait for logs to appear
            thread::sleep(Duration::from_secs(2));
            
            // Find our test containers in the watchers
            let mut found_container1 = false;
            let mut found_container2 = false;
            
            for i in 0..manager.watcher_count() {
                let watcher = manager.get_watcher(i).unwrap();
                if watcher.container_name() == container_name1 {
                    found_container1 = true;
                    assert!(!watcher.get_logs().is_empty());
                } else if watcher.container_name() == container_name2 {
                    found_container2 = true;
                    assert!(!watcher.get_logs().is_empty());
                }
            }
            
            assert!(found_container1, "Should find container1 in watchers");
            assert!(found_container2, "Should find container2 in watchers");
            
            cleanup_test_container(&container_name1);
            cleanup_test_container(&container_name2);
            Ok(())
        }

        #[test]
        fn test_docker_log_manager_refresh() -> Result<(), std::io::Error> {
            let container_name = setup_test_container()?;
            
            let mut manager = DockerLogManager::new();
            manager.start_watching_all_containers()?;
            
            let initial_count = manager.watcher_count();
            
            // Start another container
            let container_name2 = {
                let name = format!("test-container-refresh-{}", std::process::id());
                Command::new("docker")
                    .args(["run", "-d", "--name", &name, "alpine", "sh", "-c", "while true; do echo refresh test; sleep 1; done"])
                    .output()?;
                name
            };
            
            // Refresh the manager
            manager.refresh()?;
            
            assert!(manager.watcher_count() > initial_count, "Should have more watchers after refreshing");
            
            cleanup_test_container(&container_name);
            cleanup_test_container(&container_name2);
            Ok(())
        }
    }
}
// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
