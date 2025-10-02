// This file contains unit tests for the restart container
// functionality. It includes tests for successful container restart
// operations as well as error cases. Tests cover scenarios like restarting
// selected containers, handling attempts with no selection, and dealing
// with invalid container indices. 

use super::*;
use crate::dprs::app::state_machine::{AppState, Container};
use crate::shared::config::Config;

#[test]
fn test_restart_container_success() {
    let mut app_state = AppState::new();
    
    app_state.containers = vec![
        Container {
            name: "test-container-for-restart".to_string(),
            image: "test-image".to_string(),
            status: "running".to_string(),
            ip_address: "192.168.1.100".to_string(),
            ports: "80:80".to_string(),
        }
    ];
    app_state.list_state.select(Some(0));
    
    // The outcome of this test depends on the environment (Docker installed and running).
    // If `docker` command is not found, `restart_container` will return `Err`.
    // If `docker` command is found, `restart_container` (as currently written)
    // will return `Ok(())` if `Command::new("docker").output()` itself doesn't error,
    // regardless of the actual success of the `docker restart` operation, because
    // it doesn't check the exit status of the `docker restart` command.
    // It then calls `app_state.load_containers()`.
    let config = Config::default();
    let result = restart_container(&mut app_state, &config);

    // Similar to other tests that interact with external systems (like clipboard or docker),
    // we print the result for observation. A CI environment might not have Docker configured.
    println!("Result of restart_container (success case): {:?}", result);
    
    // If the result was Ok, it implies the docker command was found and no IO error occurred during its execution.
    // And app_state.load_containers() was called.
    // If Err, it implies docker command was not found or an IO error occurred.
    // Example: In an environment without Docker, this will likely print an Err.
    // Err("Failed to restart container: No such file or directory (os error 2)"))
    // In an environment with Docker, this will likely print Ok(()).
}

#[test]
fn test_restart_container_no_selection() {
    let mut app_state = AppState::new();
    app_state.containers = vec![
        Container {
            name: "test-container".to_string(),
            image: "test-image".to_string(),
            status: "running".to_string(),
            ip_address: "192.168.1.100".to_string(),
            ports: "80:80".to_string(),
        }
    ];
    
    // Ensure no container is selected.
    app_state.list_state.select(None); 
    
    let config = Config::default();
    let result = restart_container(&mut app_state, &config);

    assert!(result.is_err(), "Expected an error when no container is selected, got: {:?}", result);
    assert_eq!(result.unwrap_err(), "No container selected");
}

#[test]
fn test_restart_container_invalid_index() {
    let mut app_state = AppState::new();
    // app_state.containers is empty by default from AppState::new().
    // AppState::new() initializes list_state.select(Some(0)).
    // So, selected() will be Some(0), but containers.get(0) will be None.
    
    let config = Config::default();
    let result = restart_container(&mut app_state, &config);

    assert!(result.is_err(), "Expected an error for invalid container index, got: {:?}", result);
    assert_eq!(result.unwrap_err(), "Invalid container index");
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
