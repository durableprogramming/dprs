// Main module file for the DPRS (Docker Process Management TUI) package.
// This file re-exports core modules for Docker container management:
// - app: Application state and user actions
// - display: UI rendering and components
// - docker_log_watcher: Container log monitoring functionality
// These modules work together to create terminal-based tools for
// Docker container management and log monitoring.

mod app;
mod display;
mod docker_log_watcher;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
