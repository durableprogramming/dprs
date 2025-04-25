use std::{
    io,
    time::{Duration, Instant},
    thread,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::app::state_machine::{AppState, AppEvent};
use crate::app::actions::{copy_ip_address, open_browser, stop_container};
use crate::display::toast::ToastManager;

mod app;
mod display;

fn main() -> Result<(), io::Error>{

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal);

    Ok(())
}


fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app_state = AppState::new();
    app_state.load_containers();
    
    // Create toast manager
    let mut toast_manager = ToastManager::new();

    // Main loop
    let mut last_update = Instant::now();
    let tick_rate = Duration::from_secs(1);
    
    loop {
        terminal.draw(|f| display::draw::<B>(f, &mut app_state, &toast_manager))?;
        
        // Handle events and timing
        let timeout = tick_rate
            .checked_sub(last_update.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
            
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => app_state.next(),
                    KeyCode::Char('k') | KeyCode::Up => app_state.previous(),
                    KeyCode::Char('r') => {
                        app_state.load_containers();
                        toast_manager.show("Containers reloaded", 2000);
                    },
                    KeyCode::Char('c') => {
                        match copy_ip_address(&app_state) {
                            Ok(_) => toast_manager.show("IP address copied to clipboard", 2000),
                            Err(e) => toast_manager.show(&format!("Error: {}", e), 2000),
                        }
                    },
                    KeyCode::Char('l') => {
                        match open_browser(&app_state) {
                            Ok(_) => toast_manager.show("Opening in browser", 2000),
                            Err(e) => toast_manager.show(&format!("Error: {}", e), 2000),
                        }
                    },
                    KeyCode::Char('x') => {
                        match stop_container(&mut app_state) {
                            Ok(_) => toast_manager.show("Container stopped", 2000),
                            Err(e) => toast_manager.show(&format!("Error: {}", e), 2000),
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // Check if we need to update
        if last_update.elapsed() >= tick_rate {
            app_state.load_containers();
            last_update = Instant::now();
            
            // Check if toasts have expired
            toast_manager.check_expired();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
