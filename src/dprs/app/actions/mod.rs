// This module exports container action handlers for the Docker Process Manager TUI.
//
// It contains functions to perform operations on Docker containers like:
//
// - copy_ip: Copies container IP address to clipboard
// - open_browser: Opens container web interface in system browser
// - restart: Restarts a selected container
// - stop_container: Stops a running container
//
// These action handlers are used by the main application to respond to user input.

pub mod compose_actions;
pub mod copy_ip;
pub mod open_browser;
pub mod restart;
pub mod restart_selected;
pub mod stop_container;
pub mod stop_selected;

pub use compose_actions::{
    restart_compose_project, restart_selected_compose_projects, stop_compose_project,
    stop_selected_compose_projects,
};
pub use copy_ip::copy_ip_address;
pub use open_browser::open_browser;
pub use restart::restart_container;
pub use restart_selected::restart_selected_containers;
pub use stop_container::stop_container;
pub use stop_selected::stop_selected_containers;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
