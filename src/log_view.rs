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
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use std::collections::VecDeque;
use std::time::Instant;

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
}

impl LogView {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: VecDeque::with_capacity(max_logs),
            max_logs,
            scroll_position: 0,
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

        // Auto-scroll to bottom when new logs come in
        self.scroll_to_bottom();
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_position > 0 {
            self.scroll_position -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_position < self.logs.len().saturating_sub(1) {
            self.scroll_position += 1;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = self.logs.len().saturating_sub(1);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }

    pub fn get_scroll_position(&mut self) -> usize {
        return self.scroll_position;
    }
}

pub fn render_log_view<B: Backend>(f: &mut Frame, log_view: &LogView, area: Rect) {
    let logs = &log_view.logs;

    let log_lines: Vec<Line> = logs
        .iter()
        .map(|entry| {
            let style = match entry.level {
                LogLevel::Info => Style::default().fg(Color::Green),
                LogLevel::Warning => Style::default().fg(Color::Yellow),
                LogLevel::Error => Style::default().fg(Color::Red),
                LogLevel::Debug => Style::default().fg(Color::Blue),
            };

            Line::from(Span::styled(&entry.message, style))
        })
        .collect();

    let current_position = if logs.is_empty() {
        "No logs".to_string()
    } else {
        format!("{}/{}", log_view.scroll_position + 1, logs.len())
    };

    let log_widget = Paragraph::new(log_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Logs ({})", current_position)),
        )
        .style(Style::default())
        .wrap(Wrap { trim: false })
        .scroll((log_view.scroll_position as u16, 0));

    f.render_widget(log_widget, area);
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
