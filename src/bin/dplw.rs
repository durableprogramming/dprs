use std::{io, time::Duration, io::stdout};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use dprs::docker_log_watcher::DockerLogManager;
use dprs::display::log_tabs::{LogTabs, render_log_tabs};
use dprs::log_view::{LogView, LogLevel, render_log_view};

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    
    let mut log_manager = DockerLogManager::new();
    log_manager.start_watching_all_containers()?;
    let result = run_app(&mut terminal, &mut log_manager);

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    
    log_manager.stop_all();

    if let Err(err) = result {
        println!("Error: {}", err);
    }
    
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, log_manager: &mut DockerLogManager) -> Result<(), io::Error> {
    // Initialize log manager
    
    // Create log view for displaying logs
    let mut log_view = LogView::new(1000);
    
    // Create tabs for container selection
    let container_names: Vec<String> = (0..log_manager.watcher_count())
        .filter_map(|i| log_manager.get_watcher(i).map(|w| w.container_name().to_string()))
        .collect();
    
    let mut log_tabs = LogTabs::new(container_names);
    
    loop {
        terminal.draw(|f| {
            let size = f.size();
            
            // Create layout with tabs at top, logs below
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Tabs
                    Constraint::Min(5),     // Logs
                    Constraint::Length(3),  // Help bar
                ].as_ref())
                .split(size);
            
            // Render tabs
            render_log_tabs::<B>(f, &log_tabs, chunks[0]);
            
            // Render logs for selected container
            if let Some(active_tab) = log_tabs.index.checked_sub(0) {
                if let Some(watcher) = log_manager.get_watcher(active_tab) {
                    // Clear and refill the log view with current container logs
                    log_view = LogView::new(1000);
                    for log_line in watcher.get_logs() {
                        log_view.add_log(log_line, LogLevel::Info);
                    }
                    
                    render_log_view::<B>(f, &log_view, chunks[1]);
                }
            }
            
            // Render help bar
            let help_text = vec![
                Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(": Quit | "),
                Span::styled("←/→", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(": Switch Container | "),
                Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(": Scroll Logs | "),
                Span::styled("r", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(": Refresh Containers"),
            ];

            let help_bar = Paragraph::new(Line::from(help_text))
                .block(Block::default().borders(Borders::ALL).title("Hotkeys"));
            
            f.render_widget(help_bar, chunks[2]);
        })?;
        
        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') =>  {
                        return Ok(());
                    },
                    KeyCode::Right => log_tabs.next(),
                    KeyCode::Left => log_tabs.previous(),
                    KeyCode::Up => log_view.scroll_up(),
                    KeyCode::Down => log_view.scroll_down(),
                    KeyCode::Home => log_view.scroll_to_top(),
                    KeyCode::End => log_view.scroll_to_bottom(),
                    KeyCode::Char('r') => {
                        log_manager.refresh()?;
                        // Update tabs with new container names
                        let container_names: Vec<String> = (0..log_manager.watcher_count())
                            .filter_map(|i| log_manager.get_watcher(i).map(|w| w.container_name().to_string()))
                            .collect();
                        log_tabs = LogTabs::new(container_names);
                    },
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}
