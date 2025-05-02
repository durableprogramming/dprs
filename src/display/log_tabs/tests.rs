
use insta::assert_snapshot;
use ratatui::{backend::TestBackend, Terminal};

use super::*;

#[test]
fn test_log_tabs_render() {
    let backend = TestBackend::new(100, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let titles = vec!["container1".to_string(), "container2".to_string(), "container3".to_string()];
    let log_tabs = LogTabs::new(titles);
    
    terminal
        .draw(|f| {
            render_log_tabs::<TestBackend>(f, &log_tabs, f.area());
        })
        .unwrap();
    
    assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_log_tabs_with_selection() {
    let backend = TestBackend::new(100, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let titles = vec!["container1".to_string(), "container2".to_string(), "container3".to_string()];
    let mut log_tabs = LogTabs::new(titles);
    log_tabs.set_index(1); // Select second tab
    
    terminal
        .draw(|f| {
            render_log_tabs::<TestBackend>(f, &log_tabs, f.area());
        })
        .unwrap();
    
    assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_log_tabs_empty() {
    let backend = TestBackend::new(100, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let log_tabs = LogTabs::new(vec![]);
    
    terminal
        .draw(|f| {
            render_log_tabs::<TestBackend>(f, &log_tabs, f.area());
        })
        .unwrap();
    
    assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_log_tabs_styles() {
    let backend = TestBackend::new(100, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let titles = vec!["container1".to_string(), "container2".to_string(), "container3".to_string()];
    let log_tabs = LogTabs::new(titles);
    
    terminal
        .draw(|f| {
            render_log_tabs::<TestBackend>(f, &log_tabs, f.area());
        })
        .unwrap();
    
    // Verify output contains expected content
    let output = terminal.backend().to_string();
    assert!(output.contains("container1"));
    assert!(output.contains("container2"));
    assert!(output.contains("container3"));
    assert!(output.contains("Containers"));
}
