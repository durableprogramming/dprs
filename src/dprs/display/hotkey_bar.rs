//
// The hotkey_bar module provides a render function for displaying
// available keyboard shortcuts at the top of the TUI application. It creates
// a visually styled horizontal bar that shows all available commands with
// their corresponding keys, using different colors to distinguish key
// types and make the interface more intuitive for users. This component
// helps users understand the available interactions at a glance.

use crate::shared::config::Config;
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

fn get_key_for_action(config: &Config, action: &str, mode: &str) -> Option<String> {
    let bindings = match mode {
        "normal" => &config.keybindings.normal_mode,
        "visual" => &config.keybindings.visual_mode,
        _ => return None,
    };

    for (key, bound_action) in bindings {
        if bound_action == action {
            return Some(key.clone());
        }
    }
    None
}

pub fn render_hotkey_bar<B: Backend>(f: &mut Frame, area: Rect, config: &Config) {
    let mut help_text = Vec::new();

    // Quit
    if let Some(key) = get_key_for_action(config, "Quit", "normal") {
        help_text.push(Span::styled(
            key,
            Style::default()
                .bg(config.get_color("background_dark"))
                .fg(config.get_color("hotkey_red"))
                .add_modifier(Modifier::BOLD),
        ));
        help_text.push(Span::raw(": Quit | "));
    }

    // Navigation
    if let Some(key) = get_key_for_action(config, "SelectNext", "normal") {
        help_text.push(Span::styled(
            key,
            Style::default()
                .bg(config.get_color("background_dark"))
                .fg(config.get_color("hotkey_yellow"))
                .add_modifier(Modifier::BOLD),
        ));
        help_text.push(Span::raw(": Down | "));
    }

    if let Some(key) = get_key_for_action(config, "SelectPrevious", "normal") {
        help_text.push(Span::styled(
            key,
            Style::default()
                .bg(config.get_color("background_dark"))
                .fg(config.get_color("hotkey_yellow"))
                .add_modifier(Modifier::BOLD),
        ));
        help_text.push(Span::raw(": Up | "));
    }

    // Copy IP
    if let Some(key) = get_key_for_action(config, "CopyIp", "normal") {
        help_text.push(Span::styled(
            key,
            Style::default()
                .bg(config.get_color("background_dark"))
                .fg(config.get_color("hotkey_green"))
                .add_modifier(Modifier::BOLD),
        ));
        help_text.push(Span::raw(": Copy IP | "));
    }

    // Open Browser
    if let Some(key) = get_key_for_action(config, "OpenBrowser", "normal") {
        help_text.push(Span::styled(
            key,
            Style::default()
                .bg(config.get_color("background_dark"))
                .fg(config.get_color("hotkey_blue"))
                .add_modifier(Modifier::BOLD),
        ));
        help_text.push(Span::raw(": Open in Browser | "));
    }

    // Stop Container
    if let Some(key) = get_key_for_action(config, "StopContainer", "normal") {
        help_text.push(Span::styled(
            key,
            Style::default()
                .bg(config.get_color("background_dark"))
                .fg(config.get_color("hotkey_magenta"))
                .add_modifier(Modifier::BOLD),
        ));
        help_text.push(Span::raw(": Stop Container | "));
    }

    // Refresh/Restart
    if let Some(key) = get_key_for_action(config, "RestartContainer", "normal") {
        help_text.push(Span::styled(
            key,
            Style::default()
                .bg(config.get_color("background_dark"))
                .fg(config.get_color("hotkey_cyan"))
                .add_modifier(Modifier::BOLD),
        ));
        help_text.push(Span::raw(": Restart | "));
    }

    // Filter
    if let Some(key) = get_key_for_action(config, "EnterFilterMode", "normal") {
        help_text.push(Span::styled(
            key,
            Style::default()
                .bg(config.get_color("background_dark"))
                .fg(config.get_color("hotkey_light_blue"))
                .add_modifier(Modifier::BOLD),
        ));
        help_text.push(Span::raw(": Filter | "));
    }

    // Clear Filter
    if let Some(key) = get_key_for_action(config, "ClearFilter", "normal") {
        help_text.push(Span::styled(
            key,
            Style::default()
                .bg(config.get_color("background_dark"))
                .fg(config.get_color("hotkey_gray"))
                .add_modifier(Modifier::BOLD),
        ));
        help_text.push(Span::raw(": Clear Filter"));
    }

    let help = Paragraph::new(Line::from(help_text))
        .style(Style::default().bg(config.get_color("background_dark")))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Hotkeys")
                .style(Style::default().bg(config.get_color("background_dark"))),
        );

    f.render_widget(help, area);
}

#[cfg(test)]
mod tests;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
