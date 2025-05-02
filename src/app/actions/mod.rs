// The actions module exports functions for Docker container operations, including copying container IP addresses to the clipboard, opening container web interfaces in a browser, and stopping containers. This module serves as the central point for user-initiated interactions with the containers managed by the application.

pub mod copy_ip;
pub mod open_browser;
pub mod stop_container;

pub use copy_ip::copy_ip_address;
pub use open_browser::open_browser;
pub use stop_container::stop_container;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.