use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::state_machine::AppState;

pub fn render_container_list<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState, area: Rect) {
    let containers = &app_state.containers;

    let items: Vec<ListItem> = containers
        .iter()
        .map(|c| {
            let header = Spans::from(vec![
                Span::styled(&c.name, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" ("),
                Span::styled(&c.image, Style::default().fg(Color::Yellow)),
                Span::raw(")"),
            ]);

            let status = Spans::from(vec![
                Span::raw("Status: "),
                Span::styled(&c.status, Style::default().fg(Color::Cyan)),
            ]);

            let ip = Spans::from(vec![
                Span::raw("IP: "),
                Span::styled(&c.ip_address, Style::default().fg(Color::Blue)),
            ]);

            let ports = Spans::from(vec![
                Span::raw("Ports: "),
                Span::styled(&c.ports, Style::default().fg(Color::Magenta)),
            ]);

            ListItem::new(vec![header, status, ip, ports])
                .style(Style::default())
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Docker Containers"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app_state.list_state);
}
