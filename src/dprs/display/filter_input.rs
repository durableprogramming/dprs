use ratatui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::dprs::app::state_machine::AppState;
use crate::shared::config::Config;

pub fn render_filter_input<B: Backend>(f: &mut Frame, app_state: &AppState, area: Rect, config: &Config) {
    if !app_state.filter_mode {
        return;
    }

    // Calculate centered position for filter input box
    let popup_width = 50;
    let popup_height = 3;
    let x = area.width.saturating_sub(popup_width) / 2;
    let y = area.height.saturating_sub(popup_height) / 2;
    
    let popup_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: popup_width.min(area.width),
        height: popup_height.min(area.height),
    };

    // Clear the area and render the input box
    f.render_widget(Clear, popup_area);
    
    let input_widget = Paragraph::new(app_state.filter_text.as_str())
        .style(Style::default().fg(config.get_color("yellow")))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Filter")
                .title_alignment(Alignment::Center)
                .style(Style::default().fg(config.get_color("blue"))),
        );

    f.render_widget(input_widget, popup_area);
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.