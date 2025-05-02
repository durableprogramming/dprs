// The library module for DPRS (Docker Process Management TUI).
// This is the main library that exports modules for Docker container management:
// - app: Defines application state and actions for container operations
// - display: Provides UI components for rendering the terminal interface
// - docker_log_watcher: Implements real-time log monitoring for Docker containers
// - log_view: Manages log display and scrolling functionality
// These components work together to create a full-featured terminal interface
// for managing Docker containers and viewing their logs.

pub mod app;
pub mod display;
pub mod docker_log_watcher;
pub mod log_view;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.