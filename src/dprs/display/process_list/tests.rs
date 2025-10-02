// The tests module for the process_list component validates the
// container list display rendering. It includes tests for different
// container list states: empty lists, populated lists with multiple
// containers, lists with selection highlighting, and proper styling of
// container information. Tests use snapshot assertions to verify visual
// appearance and content checks to ensure all container details (name,
// image, status, IP, ports) are correctly displayed in the formatted output.

use insta::assert_snapshot;
use ratatui::{backend::TestBackend, Terminal};

use crate::dprs::app::state_machine::{AppState, Container};
use crate::shared::config::Config;

use super::render_container_list;

#[test]
fn test_container_list_render_empty() {
    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app_state = AppState::new();
    let config = Config::default();

    terminal
        .draw(|f| {
            render_container_list::<TestBackend>(f, &mut app_state, f.area(), &config);
        })
        .unwrap();

    assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_container_list_render_with_containers() {
    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app_state = AppState::new();
    
    // Add test containers
    app_state.containers = vec![
        Container {
            name: "web-server".to_string(),
            image: "nginx:latest".to_string(),
            status: "Up 2 hours".to_string(),
            ip_address: "172.17.0.2".to_string(),
            ports: "80/tcp, 443/tcp".to_string(),
        },
        Container {
            name: "database".to_string(),
            image: "postgres:13".to_string(),
            status: "Up 1 day".to_string(),
            ip_address: "172.17.0.3".to_string(),
            ports: "5432/tcp".to_string(),
        },
    ];
    let config = Config::default();

    terminal
        .draw(|f| {
            render_container_list::<TestBackend>(f, &mut app_state, f.area(), &config);
        })
        .unwrap();

    assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_container_list_with_selection() {
    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app_state = AppState::new();
    
    // Add test containers
    app_state.containers = vec![
        Container {
            name: "web-server".to_string(),
            image: "nginx:latest".to_string(),
            status: "Up 2 hours".to_string(),
            ip_address: "172.17.0.2".to_string(),
            ports: "80/tcp, 443/tcp".to_string(),
        },
        Container {
            name: "database".to_string(),
            image: "postgres:13".to_string(),
            status: "Up 1 day".to_string(),
            ip_address: "172.17.0.3".to_string(),
            ports: "5432/tcp".to_string(),
        },
    ];
    
    // Set selection to second item
    app_state.list_state.select(Some(1));
    let config = Config::default();

    terminal
        .draw(|f| {
            render_container_list::<TestBackend>(f, &mut app_state, f.area(), &config);
        })
        .unwrap();

    assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_container_list_styles() {
    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app_state = AppState::new();
    
    // Add a container with various fields to test styling
    app_state.containers = vec![
        Container {
            name: "test-container".to_string(),
            image: "test-image:latest".to_string(),
            status: "Up 3 minutes".to_string(),
            ip_address: "172.17.0.4".to_string(),
            ports: "8080:80/tcp".to_string(),
        },
    ];
    let config = Config::default();

    terminal
        .draw(|f| {
            render_container_list::<TestBackend>(f, &mut app_state, f.area(), &config);
        })
        .unwrap();
    
    // Verify output contains expected content
    let output = terminal.backend().to_string();
    assert!(output.contains("test-container"));
    assert!(output.contains("test-image:latest"));
    assert!(output.contains("Up 3 minutes"));
    assert!(output.contains("172.17.0.4"));
    assert!(output.contains("8080:80/tcp"));
    
    assert_snapshot!(output);
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
