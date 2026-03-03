// The commands module implements command-line interface functionality
// for executing Docker operations and navigation commands. It supports
// various container operations like stop, restart, logs, exec, and inspect,
// with flexible container specification including regex patterns,
// wildcards, and container ID matching.

use crate::dprs::app::state_machine::{AppState, Container};
use regex::Regex;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum CommandResult {
    Success(String),
    Error(String),
    Navigation(usize),
    Quit,
    ConfigReload(Box<crate::shared::config::Config>),
}

pub struct CommandExecutor {
    command_history: Vec<String>,
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            command_history: Vec::new(),
        }
    }

    pub fn execute(&mut self, command: &str, app_state: &mut AppState) -> CommandResult {
        let command = command.trim();

        if command.is_empty() {
            return CommandResult::Error("Empty command".to_string());
        }

        self.add_to_history(command.to_string());

        if command == "q" || command == "quit" {
            return CommandResult::Quit;
        }

        // Handle numeric navigation (e.g., :5, :10, :$)
        if command == "$" {
            let count = app_state.get_displayed_container_count();
            if count > 0 {
                return CommandResult::Navigation(count - 1);
            }
            return CommandResult::Error("No containers available".to_string());
        }

        if let Ok(line_num) = command.parse::<usize>() {
            let count = app_state.get_displayed_container_count();
            if line_num > 0 && line_num <= count {
                return CommandResult::Navigation(line_num - 1);
            }
            return CommandResult::Error(format!("Invalid line number: {}", line_num));
        }

        // Parse command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return CommandResult::Error("Empty command".to_string());
        }

        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "stop" => self.execute_container_command("stop", args, app_state),
            "start" => self.execute_container_command("start", args, app_state),
            "restart" => self.execute_container_command("restart", args, app_state),
            "kill" => self.execute_container_command("kill", args, app_state),
            "pause" => self.execute_container_command("pause", args, app_state),
            "unpause" => self.execute_container_command("unpause", args, app_state),
            "rm" | "remove" => self.execute_container_command("rm", args, app_state),
            "logs" => self.execute_logs_command(args, app_state),
            "exec" => self.execute_exec_command(args, app_state),
            "inspect" => self.execute_inspect_command(args, app_state),
            "build" => self.execute_build_command(args, app_state),
            "ps" | "refresh" => match app_state.refresh_containers() {
                Ok(_) => CommandResult::Success("Containers refreshed".to_string()),
                Err(e) => CommandResult::Error(format!("Failed to refresh: {}", e)),
            },
            "set" => self.execute_set_command(args, app_state),
            "reload" | "config" => self.execute_reload_command(),
            _ => CommandResult::Error(format!("Unknown command: {}", cmd)),
        }
    }

    fn execute_container_command(
        &self,
        operation: &str,
        args: &[&str],
        app_state: &AppState,
    ) -> CommandResult {
        if args.is_empty() {
            // Use currently selected container
            if let Some(container) = app_state.get_selected_container() {
                return self.docker_operation(operation, &container.name);
            } else {
                return CommandResult::Error("No container selected".to_string());
            }
        }

        let containers = app_state.get_displayed_containers();
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for arg in args {
            let matched_containers = self.resolve_container_spec(arg, &containers);

            if matched_containers.is_empty() {
                errors.push(format!("No containers found matching: {}", arg));
                continue;
            }

            for container in matched_containers {
                match self.docker_operation(operation, &container.name) {
                    CommandResult::Success(msg) => results.push(msg),
                    CommandResult::Error(err) => errors.push(err),
                    _ => {}
                }
            }
        }

        if !errors.is_empty() {
            CommandResult::Error(errors.join("; "))
        } else if !results.is_empty() {
            CommandResult::Success(results.join("; "))
        } else {
            CommandResult::Error("No operations performed".to_string())
        }
    }

    fn execute_logs_command(&self, args: &[&str], app_state: &AppState) -> CommandResult {
        let container_name = if args.is_empty() {
            if let Some(container) = app_state.get_selected_container() {
                container.name.clone()
            } else {
                return CommandResult::Error("No container selected".to_string());
            }
        } else {
            let containers = app_state.get_displayed_containers();
            let matched = self.resolve_container_spec(args[0], &containers);
            if matched.is_empty() {
                return CommandResult::Error(format!("No container found matching: {}", args[0]));
            }
            matched[0].name.clone()
        };

        match Command::new("docker")
            .args(["logs", "--tail", "100", &container_name])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let logs = String::from_utf8_lossy(&output.stdout);
                    CommandResult::Success(format!("Logs for {}:\n{}", container_name, logs))
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    CommandResult::Error(format!("Failed to get logs: {}", error))
                }
            }
            Err(e) => CommandResult::Error(format!("Failed to execute docker logs: {}", e)),
        }
    }

    fn execute_exec_command(&self, args: &[&str], app_state: &AppState) -> CommandResult {
        if args.is_empty() {
            return CommandResult::Error("Usage: :exec <container> [command]".to_string());
        }

        let containers = app_state.get_displayed_containers();
        let matched = self.resolve_container_spec(args[0], &containers);

        if matched.is_empty() {
            return CommandResult::Error(format!("No container found matching: {}", args[0]));
        }

        let container_name = &matched[0].name;
        let exec_cmd = if args.len() > 1 {
            args[1..].join(" ")
        } else {
            "/bin/bash".to_string()
        };

        CommandResult::Success(format!(
            "Would execute: docker exec -it {} {}",
            container_name, exec_cmd
        ))
    }

    fn execute_inspect_command(&self, args: &[&str], app_state: &AppState) -> CommandResult {
        let container_name = if args.is_empty() {
            if let Some(container) = app_state.get_selected_container() {
                container.name.clone()
            } else {
                return CommandResult::Error("No container selected".to_string());
            }
        } else {
            let containers = app_state.get_displayed_containers();
            let matched = self.resolve_container_spec(args[0], &containers);
            if matched.is_empty() {
                return CommandResult::Error(format!("No container found matching: {}", args[0]));
            }
            matched[0].name.clone()
        };

        match Command::new("docker")
            .args(["inspect", &container_name])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let inspect_data = String::from_utf8_lossy(&output.stdout);
                    CommandResult::Success(format!("Inspect {}:\n{}", container_name, inspect_data))
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    CommandResult::Error(format!("Failed to inspect: {}", error))
                }
            }
            Err(e) => CommandResult::Error(format!("Failed to execute docker inspect: {}", e)),
        }
    }

    fn execute_build_command(&self, args: &[&str], app_state: &AppState) -> CommandResult {
        // Get compose project from selected container or argument
        let project_name = if args.is_empty() {
            if let Some(container) = app_state.get_selected_container() {
                if let Some(ref project) = container.compose_project {
                    project.clone()
                } else {
                    return CommandResult::Error(
                        "Selected container is not part of a compose project".to_string(),
                    );
                }
            } else {
                return CommandResult::Error("No container selected".to_string());
            }
        } else {
            args[0].to_string()
        };

        // Execute docker compose build for the project
        match Command::new("docker")
            .args(["compose", "-p", &project_name, "build"])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    CommandResult::Success(format!("Built compose project: {}", project_name))
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    CommandResult::Error(format!("Failed to build {}: {}", project_name, error))
                }
            }
            Err(e) => CommandResult::Error(format!("Failed to execute docker compose build: {}", e)),
        }
    }

    fn execute_set_command(&self, args: &[&str], _app_state: &mut AppState) -> CommandResult {
        if args.is_empty() {
            return CommandResult::Success("Available settings: tabular".to_string());
        }

        match args[0] {
            "tabular" => {
                CommandResult::Success("Tabular mode settings not yet implemented".to_string())
            }
            _ => CommandResult::Error(format!("Unknown setting: {}", args[0])),
        }
    }

    fn execute_reload_command(&self) -> CommandResult {
        let config = crate::shared::config::Config::load();
        CommandResult::ConfigReload(Box::new(config))
    }

    fn resolve_container_spec(&self, spec: &str, containers: &[Container]) -> Vec<Container> {
        // Handle * to match all containers
        if spec == "*" {
            return containers.to_vec();
        }

        // Handle label matching: label:value or label:/regex/
        if spec.starts_with("label:") {
            let label_spec = &spec[6..]; // Remove "label:" prefix

            // Check if it's a regex pattern: label:/regex/
            if label_spec.starts_with('/') && label_spec.ends_with('/') && label_spec.len() > 2 {
                let pattern = &label_spec[1..label_spec.len() - 1];
                if let Ok(re) = Regex::new(pattern) {
                    return containers
                        .iter()
                        .filter(|c| {
                            c.labels.values().any(|v| re.is_match(v))
                                || c.labels.keys().any(|k| re.is_match(k))
                        })
                        .cloned()
                        .collect();
                }
            } else {
                // Exact label match (key or key=value)
                if let Some((key, value)) = label_spec.split_once('=') {
                    // Match label with specific value
                    return containers
                        .iter()
                        .filter(|c| c.labels.get(key).map_or(false, |v| v == value))
                        .cloned()
                        .collect();
                } else {
                    // Match any container with this label key
                    return containers
                        .iter()
                        .filter(|c| c.labels.contains_key(label_spec))
                        .cloned()
                        .collect();
                }
            }
        }

        // Handle regex patterns (enclosed in //)
        if spec.starts_with('/') && spec.ends_with('/') && spec.len() > 2 {
            let pattern = &spec[1..spec.len() - 1];
            if let Ok(re) = Regex::new(pattern) {
                return containers
                    .iter()
                    .filter(|c| {
                        re.is_match(&c.name)
                            || re.is_match(&c.image)
                            || re.is_match(&c.status)
                            || c.labels.values().any(|v| re.is_match(v))
                    })
                    .cloned()
                    .collect();
            }
        }

        // Handle wildcards (contains * or ?)
        if spec.contains('*') || spec.contains('?') {
            let pattern = spec.replace('*', ".*").replace('?', ".");
            if let Ok(re) = Regex::new(&format!("^{}$", pattern)) {
                return containers
                    .iter()
                    .filter(|c| re.is_match(&c.name))
                    .cloned()
                    .collect();
            }
        }

        // Exact container name or ID match
        containers
            .iter()
            .filter(|c| c.name == spec || c.name.starts_with(spec) || c.container_id.starts_with(spec))
            .cloned()
            .collect()
    }

    fn docker_operation(&self, operation: &str, container_name: &str) -> CommandResult {
        match Command::new("docker")
            .args([operation, container_name])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    CommandResult::Success(format!("{} {}", operation, container_name))
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    CommandResult::Error(format!(
                        "Failed to {} {}: {}",
                        operation, container_name, error
                    ))
                }
            }
            Err(e) => {
                CommandResult::Error(format!("Failed to execute docker {}: {}", operation, e))
            }
        }
    }

    fn add_to_history(&mut self, command: String) {
        if !self.command_history.contains(&command) {
            self.command_history.push(command);
            if self.command_history.len() > 100 {
                self.command_history.remove(0);
            }
        }
    }

    pub fn get_history(&self) -> &[String] {
        &self.command_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_container(
        name: &str,
        image: &str,
        labels: Vec<(&str, &str)>,
    ) -> Container {
        let mut label_map = HashMap::new();
        for (k, v) in labels {
            label_map.insert(k.to_string(), v.to_string());
        }

        Container {
            name: name.to_string(),
            image: image.to_string(),
            status: "Up".to_string(),
            ip_address: "127.0.0.1".to_string(),
            ports: "8080".to_string(),
            cpu_usage: "0%".to_string(),
            memory_usage: "0MiB".to_string(),
            image_hash: "abc123".to_string(),
            container_id: "container123".to_string(),
            started_at: "2025-01-01T00:00:00Z".to_string(),
            compose_project: Some("test-project".to_string()),
            labels: label_map,
        }
    }

    #[test]
    fn test_resolve_wildcard_all() {
        let executor = CommandExecutor::new();
        let containers = vec![
            create_test_container("web-1", "nginx", vec![]),
            create_test_container("api-1", "node", vec![]),
            create_test_container("db-1", "postgres", vec![]),
        ];

        let result = executor.resolve_container_spec("*", &containers);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_resolve_label_exact_key() {
        let executor = CommandExecutor::new();
        let containers = vec![
            create_test_container("web-1", "nginx", vec![("env", "prod")]),
            create_test_container("api-1", "node", vec![("env", "dev")]),
            create_test_container("db-1", "postgres", vec![]),
        ];

        let result = executor.resolve_container_spec("label:env", &containers);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|c| c.name == "web-1"));
        assert!(result.iter().any(|c| c.name == "api-1"));
    }

    #[test]
    fn test_resolve_label_key_value() {
        let executor = CommandExecutor::new();
        let containers = vec![
            create_test_container("web-1", "nginx", vec![("env", "prod")]),
            create_test_container("api-1", "node", vec![("env", "dev")]),
            create_test_container("db-1", "postgres", vec![]),
        ];

        let result = executor.resolve_container_spec("label:env=prod", &containers);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "web-1");
    }

    #[test]
    fn test_resolve_label_regex() {
        let executor = CommandExecutor::new();
        let containers = vec![
            create_test_container("web-1", "nginx", vec![("env", "production")]),
            create_test_container("api-1", "node", vec![("env", "development")]),
            create_test_container("db-1", "postgres", vec![("env", "test")]),
        ];

        let result = executor.resolve_container_spec("label:/prod/", &containers);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "web-1");
    }

    #[test]
    fn test_resolve_regex_pattern() {
        let executor = CommandExecutor::new();
        let containers = vec![
            create_test_container("web-1", "nginx", vec![]),
            create_test_container("web-2", "nginx", vec![]),
            create_test_container("api-1", "node", vec![]),
        ];

        let result = executor.resolve_container_spec("/^web-/", &containers);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|c| c.name == "web-1"));
        assert!(result.iter().any(|c| c.name == "web-2"));
    }

    #[test]
    fn test_resolve_wildcard_pattern() {
        let executor = CommandExecutor::new();
        let containers = vec![
            create_test_container("web-1", "nginx", vec![]),
            create_test_container("web-2", "nginx", vec![]),
            create_test_container("api-1", "node", vec![]),
        ];

        let result = executor.resolve_container_spec("web-*", &containers);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|c| c.name == "web-1"));
        assert!(result.iter().any(|c| c.name == "web-2"));
    }

    #[test]
    fn test_resolve_exact_name() {
        let executor = CommandExecutor::new();
        let containers = vec![
            create_test_container("web-1", "nginx", vec![]),
            create_test_container("api-1", "node", vec![]),
        ];

        let result = executor.resolve_container_spec("web-1", &containers);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "web-1");
    }

    #[test]
    fn test_resolve_container_id_prefix() {
        let executor = CommandExecutor::new();
        let mut containers = vec![create_test_container("web-1", "nginx", vec![])];
        containers[0].container_id = "abc123def456".to_string();

        let result = executor.resolve_container_spec("abc123", &containers);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "web-1");
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
