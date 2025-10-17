// The log_tabs module provides UI components for switching between
// Docker container logs. It implements a tabbed interface that allows
// users to select which container's logs to view. The module contains
// the LogTabs struct that manages tab state and navigation, along with
// a render function that displays the tabs at the top of the log viewer
// with proper styling and highlighting of the currently selected container.

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Tabs},
    Frame,
};
use crate::shared::config::Config;

pub struct LogTabs {
    pub titles: Vec<String>,
    pub index: usize,
}

impl LogTabs {
    pub fn new(titles: Vec<String>) -> Self {
        Self { titles, index: 0 }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }

    pub fn set_index(&mut self, index: usize) {
        if index < self.titles.len() {
            self.index = index;
        }
    }

    pub fn get_active_tab_name(&self) -> Option<&String> {
        self.titles.get(self.index)
    }
}

pub fn render_log_tabs<B: Backend>(f: &mut Frame, log_tabs: &LogTabs, area: Rect, config: &Config) {
    let titles: Vec<Line> = log_tabs
        .titles
        .iter()
        .map(|t| Line::from(vec![Span::styled(t, Style::default().bg(config.get_color("background_main")).fg(config.get_color("text_main")))]))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Containers")
                .border_type(BorderType::Rounded),
        )
        .select(log_tabs.index)
        .style(Style::default().bg(config.get_color("background_main")).fg(config.get_color("text_main")))
        .highlight_style(
            Style::default()
                .bg(config.get_color("background_main"))
                .fg(config.get_color("message_warning"))
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}



// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
