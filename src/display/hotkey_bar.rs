//
// The hotkey_bar module provides a render function for displaying
// available keyboard shortcuts at the top of the TUI application. It creates
// a visually styled horizontal bar that shows all available commands with
// their corresponding keys, using different colors to distinguish key
// types and make the interface more intuitive for users. This component
// helps users understand the available interactions at a glance.

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_hotkey_bar<B: Backend>(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Span::styled(
            "q",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Quit | "),
        Span::styled(
            "j/↓",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Down | "),
        Span::styled(
            "k/↑",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Up | "),
        Span::styled(
            "c",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Copy IP | "),
        Span::styled(
            "l",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Open in Browser | "),
        Span::styled(
            "x",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Stop Container | "),
        Span::styled(
            "r",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Restart | "),
        Span::styled(
            "/",
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Filter | "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Clear Filter"),
    ];

    let help = Paragraph::new(Line::from(help_text))
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Hotkeys"));

    f.render_widget(help, area);
}

#[cfg(test)]
mod tests;

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
