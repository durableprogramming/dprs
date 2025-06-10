//  The tests module for the hotkey_bar component validates the rendering
//  and content of the keyboard shortcut display. It contains two test cases:
//  one that verifies the visual appearance of the rendered hotkey bar using
//  snapshot testing, and another that checks all key commands are properly
//  included in the output. These tests ensure the hotkey help display is
//  complete and correctly formatted.

use insta::assert_snapshot;
use ratatui::{backend::TestBackend, Terminal};

use super::render_hotkey_bar;

#[test]
fn test_hotkey_bar_render() {
    let backend = TestBackend::new(100, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    
    terminal
        .draw(|f| {
            render_hotkey_bar::<TestBackend>(f, f.area());
        })
        .unwrap();
    
    assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_hotkey_bar_contents() {
    let backend = TestBackend::new(100, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    
    terminal
        .draw(|f| {
            render_hotkey_bar::<TestBackend>(f, f.area());
        })
        .unwrap();
    
    let output = terminal.backend().to_string();
    
    // Verify all key commands are included
    assert!(output.contains("q: Quit"));
    assert!(output.contains("j/↓: Down"));
    assert!(output.contains("k/↑: Up"));
    assert!(output.contains("c: Copy IP"));
    assert!(output.contains("l: Open in Browser"));
    assert!(output.contains("x: Stop Container"));
    assert!(output.contains("r: Reload"));
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
