// The App module defines the application state and logic. It contains two sub-modules:
// - actions: handles user interactions like copying IP addresses, opening browsers, and stopping containers
// - state_machine: manages the application state, including container data and selection state

pub mod actions;
pub mod state_machine;

pub use state_machine::{AppEvent, AppState, Container};

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
