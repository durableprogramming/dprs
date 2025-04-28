use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Tabs, BorderType},
    Frame,
};

pub struct LogTabs {
    pub titles: Vec<String>,
    pub index: usize,
}

impl LogTabs {
    pub fn new(titles: Vec<String>) -> Self {
        Self {
            titles,
            index: 0,
        }
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
        .map(|t| {
            Line::from(vec![Span::styled(
                t,
                Style::default().fg(Color::White),
            )])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Containers").border_type(BorderType::Rounded))
        .select(log_tabs.index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}
