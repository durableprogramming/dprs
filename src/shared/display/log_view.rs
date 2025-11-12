// The log_view module provides components for displaying and navigating
// container logs in the Docker Process Management TUI. It defines the
// LogView struct for storing and managing log entries with different
// severity levels (Info, Warning, Error, Debug), along with methods
// for scrolling through logs and adding new entries. The module also
// implements a renderer function that creates a styled paragraph widget
// for displaying logs with appropriate colors and formatting based on log
// levels, supporting features like auto-scrolling and position tracking.

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::shared::config::Config;
use ansi_to_tui::IntoText;
use std::collections::VecDeque;
use std::time::Instant;
use tailspin::Highlighter;

pub struct LogEntry {
    pub timestamp: Instant,
    pub message: String,
    pub level: LogLevel,
}

pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

pub struct LogView {
    logs: VecDeque<LogEntry>,
    max_logs: usize,
    scroll_position: usize,
    follow_mode: bool,
}

impl LogView {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: VecDeque::with_capacity(max_logs),
            max_logs,
            scroll_position: 0,
            follow_mode: true,
        }
    }

    pub fn add_log(&mut self, message: String, level: LogLevel) {
        let log_entry = LogEntry {
            timestamp: Instant::now(),
            message,
            level,
        };

        self.logs.push_back(log_entry);

        // Trim logs if we exceed max capacity
        if self.logs.len() > self.max_logs {
            self.logs.pop_front();
        }

        // Auto-scroll to bottom when new logs come in if follow mode is enabled
        if self.follow_mode {
            self.scroll_to_bottom();
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_position > 0 {
            self.scroll_position -= 1;
            self.follow_mode = false;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_position < self.logs.len().saturating_sub(1) {
            self.scroll_position += 1;
            self.follow_mode = false;
        }
    }

    pub fn page_up(&mut self, visible_height: usize) {
        if self.scroll_position > visible_height {
            self.scroll_position -= visible_height;
        } else {
            self.scroll_position = 0;
        }
        self.follow_mode = false;
    }

    pub fn page_down(&mut self, visible_height: usize) {
        let max_scroll = self.logs.len().saturating_sub(1);
        self.scroll_position = (self.scroll_position + visible_height).min(max_scroll);
        self.follow_mode = false;
    }

    pub fn enable_follow(&mut self) {
        self.follow_mode = true;
        self.scroll_to_bottom();
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = self.logs.len().saturating_sub(1);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }

    pub fn get_scroll_position(&mut self) -> usize {
        self.scroll_position
    }

    pub fn get_log_count(&self) -> usize {
        self.logs.len()
    }

    pub fn set_scroll_position(&mut self, position: usize) {
        self.scroll_position = position.min(self.logs.len().saturating_sub(1));
    }
}

fn parse_log_with_tailspin<'a>(message: &'a str, config: &'a Config) -> Vec<Span<'a>> {
    // Create a tailspin highlighter with default settings
    let highlighter = Highlighter::default();

    // Apply tailspin highlighting to get ANSI-colored string
    let highlighted_message = highlighter.apply(message);

    // Convert ANSI-colored string to ratatui Text and extract spans
    match highlighted_message.as_ref().into_text() {
        Ok(text) => {
            // Extract spans from the first line of the text
            if let Some(line) = text.lines.first() {
                line.spans.clone()
            } else {
                // Fallback to plain text if conversion fails
                vec![Span::styled(
                    message,
                    Style::default()
                        .bg(config.get_color("background_main"))
                        .fg(config.get_color("text_main")),
                )]
            }
        }
        Err(_) => {
            // Fallback to plain text if conversion fails
            vec![Span::styled(
                message,
                Style::default()
                    .bg(config.get_color("background_main"))
                    .fg(config.get_color("text_main")),
            )]
        }
    }
}

pub fn render_log_view<B: Backend>(f: &mut Frame, log_view: &LogView, area: Rect, config: &Config) {
    let logs = &log_view.logs;

    let log_lines: Vec<Line> = logs
        .iter()
        .map(|entry| {
            let spans = parse_log_with_tailspin(&entry.message, config);
            Line::from(spans)
        })
        .collect();

    let current_position = if logs.is_empty() {
        "No logs".to_string()
    } else {
        format!("{}/{}", log_view.scroll_position + 1, logs.len())
    };

    // Calculate the actual number of wrapped lines up to scroll_position
    // We need to account for text wrapping in the available width
    let available_width = area.width.saturating_sub(2) as usize; // Account for borders
    let mut wrapped_lines_before_scroll = 0;

    for (idx, entry) in logs.iter().enumerate() {
        if idx >= log_view.scroll_position {
            break;
        }
        // Calculate how many visual lines this log entry will take when wrapped
        let message_width = entry.message.chars().count();
        let lines_needed = if available_width > 0 {
            message_width.div_ceil(available_width)
        } else {
            1
        };
        wrapped_lines_before_scroll += lines_needed.max(1);
    }

    let log_widget = Paragraph::new(log_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Logs ({})", current_position)),
        )
        .style(Style::default().bg(config.get_color("background_main")))
        .wrap(Wrap { trim: false })
        .scroll((wrapped_lines_before_scroll as u16, 0));

    f.render_widget(log_widget, area);
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
