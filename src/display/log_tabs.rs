// The log_tabs module provides UI components for switching between Docker container logs.
// It implements a tabbed interface that allows users to select which container's logs to view.
// The module contains the LogTabs struct that manages tab state and navigation, along with
// a render function that displays the tabs at the top of the log viewer with proper styling
// and highlighting of the currently selected container.

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Tabs},
    Frame,
};

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

pub fn render_log_tabs<B: Backend>(f: &mut Frame, log_tabs: &LogTabs, area: Rect) {
    let titles: Vec<Line> = log_tabs
        .titles
        .iter()
        .map(|t| Line::from(vec![Span::styled(t, Style::default().fg(Color::White))]))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Containers")
                .border_type(BorderType::Rounded),
        )
        .select(log_tabs.index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_tabs_new() {
        let titles = vec!["container1".to_string(), "container2".to_string()];
        let tabs = LogTabs::new(titles.clone());
        
        assert_eq!(tabs.titles, titles);
        assert_eq!(tabs.index, 0);
    }

    #[test]
    fn test_log_tabs_next() {
        let titles = vec!["container1".to_string(), "container2".to_string(), "container3".to_string()];
        let mut tabs = LogTabs::new(titles);
        
        assert_eq!(tabs.index, 0);
        
        tabs.next();
        assert_eq!(tabs.index, 1);
        
        tabs.next();
        assert_eq!(tabs.index, 2);
        
        tabs.next();
        assert_eq!(tabs.index, 0); // Should wrap around
    }

    #[test]
    fn test_log_tabs_previous() {
        let titles = vec!["container1".to_string(), "container2".to_string(), "container3".to_string()];
        let mut tabs = LogTabs::new(titles);
        
        assert_eq!(tabs.index, 0);
        
        tabs.previous();
        assert_eq!(tabs.index, 2); // Should wrap to end
        
        tabs.previous();
        assert_eq!(tabs.index, 1);
        
        tabs.previous();
        assert_eq!(tabs.index, 0);
    }

    #[test]
    fn test_set_index() {
        let titles = vec!["container1".to_string(), "container2".to_string()];
        let mut tabs = LogTabs::new(titles);
        
        assert_eq!(tabs.index, 0);
        
        tabs.set_index(1);
        assert_eq!(tabs.index, 1);
        
        // Test setting index out of bounds - should not change
        tabs.set_index(5);
        assert_eq!(tabs.index, 1);
    }

    #[test]
    fn test_get_active_tab_name() {
        let titles = vec!["container1".to_string(), "container2".to_string()];
        let mut tabs = LogTabs::new(titles);
        
        assert_eq!(tabs.get_active_tab_name(), Some(&"container1".to_string()));
        
        tabs.next();
        assert_eq!(tabs.get_active_tab_name(), Some(&"container2".to_string()));
        
        // Test with empty titles
        let empty_tabs = LogTabs::new(vec![]);
        assert_eq!(empty_tabs.get_active_tab_name(), None);
    }
}
// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
