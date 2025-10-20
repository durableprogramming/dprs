// The modes module defines the modal architecture for the DPRS application,
// implementing Vim-like modes including Normal, Visual, and Command modes.
// This provides different interaction contexts and keybindings based on
// the current mode state.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Visual,
    Command,
    Search,
}

impl Mode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Visual => "VISUAL",
            Mode::Command => "COMMAND",
            Mode::Search => "SEARCH",
        }
    }
}

#[derive(Debug, Clone)]
pub struct VisualSelection {
    pub start_index: usize,
    pub current_index: usize,
    pub selected_indices: HashSet<usize>,
}

impl VisualSelection {
    pub fn new(start_index: usize) -> Self {
        let mut selected_indices = HashSet::new();
        selected_indices.insert(start_index);

        Self {
            start_index,
            current_index: start_index,
            selected_indices,
        }
    }

    pub fn extend_to(&mut self, index: usize) {
        self.current_index = index;
        self.selected_indices.clear();

        let start = self.start_index.min(index);
        let end = self.start_index.max(index);

        for i in start..=end {
            self.selected_indices.insert(i);
        }
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.selected_indices.contains(&index)
    }
}

#[derive(Debug, Clone)]
pub struct CommandState {
    pub input: String,
    pub cursor_pos: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
}

impl Default for CommandState {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            history: Vec::new(),
            history_index: None,
        }
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_pos = 0;
        self.history_index = None;
    }

    pub fn add_to_history(&mut self, command: String) {
        if !command.is_empty() && !self.history.contains(&command) {
            self.history.push(command);
            if self.history.len() > 100 {
                self.history.remove(0);
            }
        }
    }

    pub fn navigate_history(&mut self, up: bool) {
        if self.history.is_empty() {
            return;
        }

        if up {
            match self.history_index {
                None => {
                    self.history_index = Some(self.history.len() - 1);
                    self.input = self.history[self.history.len() - 1].clone();
                }
                Some(index) if index > 0 => {
                    self.history_index = Some(index - 1);
                    self.input = self.history[index - 1].clone();
                }
                _ => {} // Already at oldest
            }
        } else {
            match self.history_index {
                Some(index) if index < self.history.len() - 1 => {
                    self.history_index = Some(index + 1);
                    self.input = self.history[index + 1].clone();
                }
                Some(_) => {
                    self.history_index = None;
                    self.input.clear();
                }
                None => {} // Already at newest
            }
        }

        self.cursor_pos = self.input.len();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    false
                } else {
                    self.input.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                    self.history_index = None;
                    true
                }
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.input.remove(self.cursor_pos);
                    self.history_index = None;
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input.len() {
                    self.input.remove(self.cursor_pos);
                    self.history_index = None;
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
                true
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.input.len();
                true
            }
            KeyCode::Up => {
                self.navigate_history(true);
                true
            }
            KeyCode::Down => {
                self.navigate_history(false);
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub is_forward: bool,
    pub current_match: Option<usize>,
    pub matches: Vec<usize>,
    pub last_query: String,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            is_forward: true,
            current_match: None,
            matches: Vec::new(),
            last_query: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.current_match = None;
        self.matches.clear();
    }

    pub fn set_query(&mut self, query: String, is_forward: bool) {
        self.query = query;
        self.is_forward = is_forward;
        self.matches.clear();
        self.current_match = None;
    }

    pub fn next_match(&mut self) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }

        match self.current_match {
            None => {
                self.current_match = Some(0);
                Some(self.matches[0])
            }
            Some(index) => {
                let next_index = if self.is_forward {
                    (index + 1) % self.matches.len()
                } else if index == 0 {
                    self.matches.len() - 1
                } else {
                    index - 1
                };
                self.current_match = Some(next_index);
                Some(self.matches[next_index])
            }
        }
    }

    pub fn update_matches(&mut self, matches: Vec<usize>) {
        self.matches = matches;
        if !self.matches.is_empty() && self.current_match.is_none() {
            self.current_match = Some(0);
        } else if self.matches.is_empty() {
            self.current_match = None;
        }
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
