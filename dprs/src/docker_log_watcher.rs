use std::{
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub struct DockerLogWatcher {
    container_id: String,
    container_name: String,
    log_lines: Arc<Mutex<Vec<String>>>,
    running: Arc<Mutex<bool>>,
}

impl DockerLogWatcher {
    pub fn new(container_id: String, container_name: String) -> Self {
        Self {
            container_id,
            container_name,
            log_lines: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(true)),
        }
    }

    pub fn start_watching(&self) -> thread::JoinHandle<()> {
        let container_id = self.container_id.clone();
        let log_lines = Arc::clone(&self.log_lines);
        let running = Arc::clone(&self.running);

        // Initialize with existing logs
        Self::load_existing_logs(&container_id, &log_lines);

        // Spawn a thread to watch for new logs
        thread::spawn(move || {
            let log_process = match Command::new("docker")
                .args(["logs", "--follow", "--tail", "0", &container_id])
                .stdout(Stdio::piped())
                .spawn() {
                Ok(process) => process,
                Err(e) => {
                    eprintln!("Failed to start log watching: {}", e);
                    return;
                }
            };

            let stdout = match log_process.stdout {
                Some(stdout) => stdout,
                None => {
                    eprintln!("Failed to open stdout from log process");
                    return;
                }
            };

            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    // Check if we should stop watching
                    if !*running.lock().unwrap() {
                        break;
                    }

                    // Add the new line to our log buffer
                    let mut logs = log_lines.lock().unwrap();
                    logs.push(line);
                }
            }
        })
    }

    fn load_existing_logs(container_id: &str, log_lines: &Arc<Mutex<Vec<String>>>) {
        match Command::new("docker")
            .args(["logs", "--tail", "100", container_id])
            .output() {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut logs = log_lines.lock().unwrap();
                for line in output_str.lines() {
                    logs.push(line.to_string());
                }
            }
            Err(e) => {
                eprintln!("Failed to get existing logs: {}", e);
            }
        }
    }

    pub fn stop_watching(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }

    pub fn get_logs(&self) -> Vec<String> {
        let logs = self.log_lines.lock().unwrap();
        logs.clone()
    }

    pub fn container_name(&self) -> &str {
        &self.container_name
    }

    pub fn container_id(&self) -> &str {
        &self.container_id
    }
}

pub struct DockerLogManager {
    watchers: Vec<DockerLogWatcher>,
    handles: Vec<thread::JoinHandle<()>>,
}

impl DockerLogManager {
    pub fn new() -> Self {
        Self {
            watchers: Vec::new(),
            handles: Vec::new(),
        }
    }

    pub fn start_watching_all_containers(&mut self) -> io::Result<()> {
        // Get all running containers
        let output = Command::new("docker")
            .args(["ps", "--format", "{{.ID}}|{{.Names}}"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Stop any existing watchers
        self.stop_all();
        self.watchers.clear();
        self.handles.clear();

        // Create new watchers for each container
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 2 {
                let container_id = parts[0].to_string();
                let container_name = parts[1].to_string();
                
                let watcher = DockerLogWatcher::new(container_id, container_name);
                let handle = watcher.start_watching();
                
                self.watchers.push(watcher);
                self.handles.push(handle);
            }
        }

        Ok(())
    }

    pub fn get_watcher(&self, index: usize) -> Option<&DockerLogWatcher> {
        self.watchers.get(index)
    }

    pub fn stop_all(&mut self) {
        for watcher in &self.watchers {
            watcher.stop_watching();
        }
        
        // Wait for all threads to finish
        while let Some(handle) = self.handles.pop() {
            // Use a timeout to avoid blocking indefinitely
            let _ = handle.join();
        }
    }

    pub fn watcher_count(&self) -> usize {
        self.watchers.len()
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        self.start_watching_all_containers()
    }
}

impl Drop for DockerLogManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}
