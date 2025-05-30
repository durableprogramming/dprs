// This module exports container action handlers for the Docker Process Manager TUI.
// It contains functions to perform operations on Docker containers like:
// - copy_ip: Copies container IP address to clipboard
// - open_browser: Opens container web interface in system browser
// - restart: Restarts a selected container
// - stop_container: Stops a running container
// These action handlers are used by the main application to respond to user input.

pub mod copy_ip;
pub mod open_browser;
pub mod restart;
pub mod stop_container;

pub use copy_ip::copy_ip_address;
pub use open_browser::open_browser;
pub use restart::restart_container;
pub use stop_container::stop_container;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
