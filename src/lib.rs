// The library module for DPRS (Docker Process Management TUI).
//
// This library is structured with shared modules that can be used by both
// dprs (Docker Process Manager) and dplw (Docker Process Log Watcher) applications:
//
// - shared: Common functionality used by both applications
// - dprs: Specific functionality for the container management TUI
// - dplw: Specific functionality for the log watching TUI
//
// The shared modules include Docker integration, input handling, display components,
// and configuration management. This structure promotes code reuse while keeping
// application-specific logic separate.

pub mod dplw;
pub mod dprs;
pub mod shared;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
