use std::{io, time::Duration, io::stdout};
use std::error::Error;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use dprs::app::state_machine::{AppState, AppEvent};
use dprs::app::actions::{copy_ip_address, open_browser, stop_container};
use dprs::display::toast::ToastManager;
use dprs::display;

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
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // App state
    let mut app_state = AppState::new();
    app_state.load_containers();
    
    // Toast notification manager
    let mut toast_manager = ToastManager::new();

    // Main event loop
    loop {
        // Draw the UI
        terminal.draw(|f| display::draw::<B>(f, &mut app_state, &toast_manager))?;
        
        // Check if toast has expired
        toast_manager.check_expired();

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => app_state.next(),
                    KeyCode::Char('k') | KeyCode::Up => app_state.previous(),
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
                    KeyCode::Char('r') => {
                        app_state.load_containers();
                        toast_manager.show("Containers reloaded", 1000);
                    },
                    _ => {}
                }
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    
    Ok(())
}
