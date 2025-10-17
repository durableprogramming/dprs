// The config module implements persistent configuration management for DPRS,
// supporting user-customizable settings including key bindings, color schemes,
// and layout options. Configuration is stored in ~/.dprs/config and loaded
// at startup with fallback to sensible defaults.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub keybindings: KeyBindings,
    pub colors: ColorConfig,
    pub layout: LayoutConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub tabular_mode: bool,
    pub auto_refresh_interval: u64, // seconds, 0 = disabled
    pub max_history_items: usize,
    #[serde(default = "default_experimental_fx")]
    pub experimental_fx: bool,
}

fn default_experimental_fx() -> bool {
    std::env::var("EXPERIMENTAL_FX").is_ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    /// Key bindings for normal mode. These are the default key mappings.
    pub normal_mode: HashMap<String, String>,
    /// Key bindings for visual mode. Keys can be:
    /// - Mapped to different actions than normal mode
    /// - Set to empty string ("") to unmap/disable them in visual mode
    /// - Added as visual-mode-only bindings (not present in normal_mode)
    pub visual_mode: HashMap<String, String>,
    pub command_mode: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub theme: String,
    pub custom_colors: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub show_headers: bool,
    pub column_widths: HashMap<String, u16>,
    pub show_borders: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut normal_mode = HashMap::new();
        
        // Navigation
        normal_mode.insert("j".to_string(), "SelectNext".to_string());
        normal_mode.insert("k".to_string(), "SelectPrevious".to_string());
        normal_mode.insert("gg".to_string(), "GoToFirst".to_string());
        normal_mode.insert("G".to_string(), "GoToLast".to_string());
        normal_mode.insert("w".to_string(), "WordNext".to_string());
        normal_mode.insert("b".to_string(), "WordPrevious".to_string());
        normal_mode.insert("Ctrl+u".to_string(), "HalfPageUp".to_string());
        normal_mode.insert("Ctrl+d".to_string(), "HalfPageDown".to_string());
        
        // Mode switching
        normal_mode.insert("v".to_string(), "EnterVisualMode".to_string());
        normal_mode.insert(":".to_string(), "EnterCommandMode".to_string());
        normal_mode.insert("/".to_string(), "EnterSearchForward".to_string());
        normal_mode.insert("?".to_string(), "EnterSearchBackward".to_string());
        
        // Search navigation
        normal_mode.insert("n".to_string(), "NextSearchResult".to_string());
        normal_mode.insert("N".to_string(), "PreviousSearchResult".to_string());
        
        // Container actions
        normal_mode.insert("s".to_string(), "StopContainer".to_string());
        normal_mode.insert("r".to_string(), "RestartContainer".to_string());
        normal_mode.insert("c".to_string(), "CopyIp".to_string());
        normal_mode.insert("o".to_string(), "OpenBrowser".to_string());
        normal_mode.insert("t".to_string(), "ToggleTabular".to_string());
        
        // Filter
        normal_mode.insert("f".to_string(), "EnterFilterMode".to_string());
        normal_mode.insert("Escape".to_string(), "ClearFilter".to_string());
        
        // Quit
        normal_mode.insert("q".to_string(), "Quit".to_string());

        let mut visual_mode = HashMap::new();
        // Navigation in visual mode (extends selection)
        visual_mode.insert("j".to_string(), "ExtendSelectionNext".to_string());
        visual_mode.insert("k".to_string(), "ExtendSelectionPrevious".to_string());

        // Container actions for selected containers
        visual_mode.insert("s".to_string(), "StopSelectedContainers".to_string());
        visual_mode.insert("r".to_string(), "RestartSelectedContainers".to_string());

        // Mode switching
        visual_mode.insert("Escape".to_string(), "EnterNormalMode".to_string());

        // Example of visual-mode-only binding (uncomment to enable):
        // visual_mode.insert("d".to_string(), "SomeVisualOnlyAction".to_string());

        // Example of unmapping a key in visual mode (set to empty string):
        // visual_mode.insert("q".to_string(), "".to_string()); // Unmap quit in visual mode

        let mut command_mode = HashMap::new();
        command_mode.insert("Enter".to_string(), "ExecuteCommand".to_string());
        command_mode.insert("Escape".to_string(), "CancelCommand".to_string());
        command_mode.insert("Tab".to_string(), "TabComplete".to_string());

        let mut custom_colors = HashMap::new();
        // Selection and visual mode colors
        custom_colors.insert("selected_bg".to_string(), "#4a4af7".to_string());
        custom_colors.insert("visual_bg".to_string(), "#5D2F00".to_string());
        custom_colors.insert("search_highlight".to_string(), "#4D2D5F".to_string());

        // Container/process list colors
        custom_colors.insert("container_name".to_string(), "#00AA00".to_string());
        custom_colors.insert("container_image".to_string(), "#AAAA00".to_string());
        custom_colors.insert("container_status".to_string(), "#00AAAA".to_string());
        custom_colors.insert("container_ip".to_string(), "#0000AA".to_string());
        custom_colors.insert("container_ports".to_string(), "#AA00AA".to_string());

        // Tabular view specific colors
        custom_colors.insert("container_image_tabular".to_string(), "#00AAAA".to_string());
        custom_colors.insert("container_status_tabular".to_string(), "#0000AA".to_string());
        custom_colors.insert("container_ip_tabular".to_string(), "#AA00AA".to_string());
        custom_colors.insert("container_ports_tabular".to_string(), "#CCCCCC".to_string());

        // Mode indicator colors
        custom_colors.insert("mode_normal".to_string(), "#00AA00".to_string());
        custom_colors.insert("mode_visual".to_string(), "#AAAA00".to_string());
        custom_colors.insert("mode_command".to_string(), "#0000AA".to_string());
        custom_colors.insert("mode_search".to_string(), "#AA00AA".to_string());

        // Hotkey colors
        custom_colors.insert("hotkey_red".to_string(), "#AA0000".to_string());
        custom_colors.insert("hotkey_yellow".to_string(), "#AAAA00".to_string());
        custom_colors.insert("hotkey_green".to_string(), "#00AA00".to_string());
        custom_colors.insert("hotkey_blue".to_string(), "#0000AA".to_string());
        custom_colors.insert("hotkey_magenta".to_string(), "#AA00AA".to_string());
        custom_colors.insert("hotkey_cyan".to_string(), "#00AAAA".to_string());
        custom_colors.insert("hotkey_white".to_string(), "#CCCCCC".to_string());
        custom_colors.insert("hotkey_gray".to_string(), "#666666".to_string());
        custom_colors.insert("hotkey_light_blue".to_string(), "#6699CC".to_string());

        // Background colors
        custom_colors.insert("background_main".to_string(), "#000000".to_string());
        custom_colors.insert("background_dark".to_string(), "#0F0F0F".to_string());
        custom_colors.insert("background_table".to_string(), "#0F0F0F".to_string());
        custom_colors.insert("background_selection".to_string(), "#1F1F1F".to_string());
        custom_colors.insert("background_alt".to_string(), "#0F0F0F".to_string());
        custom_colors.insert("background_selection_orange".to_string(), "#2F1F0F".to_string());
        custom_colors.insert("background_very_dark".to_string(), "#0A0A0A".to_string());
        custom_colors.insert("background_alt_dark".to_string(), "#0A0A0A".to_string());

        // Border colors
        custom_colors.insert("border_main".to_string(), "#00AAAA".to_string());
        custom_colors.insert("border_light".to_string(), "#4A9EFF".to_string());

        // Text colors
        custom_colors.insert("text_selection".to_string(), "#8080FF".to_string());
        custom_colors.insert("text_main".to_string(), "#FFFFFF".to_string());

        // Message colors
        custom_colors.insert("message_error".to_string(), "#AA0000".to_string());
        custom_colors.insert("message_warning".to_string(), "#AAAA00".to_string());
        custom_colors.insert("message_success".to_string(), "#00AA00".to_string());

        // Filter colors
        custom_colors.insert("filter_text".to_string(), "#FFFF00".to_string());
        custom_colors.insert("filter_cursor".to_string(), "#0000FF".to_string());

        let mut column_widths = HashMap::new();
        column_widths.insert("name".to_string(), 25);
        column_widths.insert("image".to_string(), 20);
        column_widths.insert("status".to_string(), 15);
        column_widths.insert("ip".to_string(), 15);
        column_widths.insert("ports".to_string(), 20);

        Self {
            general: GeneralConfig {
                tabular_mode: false,
                auto_refresh_interval: 0,
                max_history_items: 100,
                experimental_fx: default_experimental_fx(),
            },
            keybindings: KeyBindings {
                normal_mode,
                visual_mode,
                command_mode,
            },
            colors: ColorConfig {
                theme: "default".to_string(),
                custom_colors,
            },
            layout: LayoutConfig {
                show_headers: true,
                column_widths,
                show_borders: true,
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_file_path();

        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match toml::from_str::<Config>(&content) {
                    Ok(mut config) => {
                        // Merge default colors with loaded colors
                        let default_config = Self::default();
                        for (key, value) in default_config.colors.custom_colors {
                            config.colors.custom_colors.entry(key).or_insert(value);
                        }
                        config
                    }
                    Err(e) => {
                        eprintln!("Error parsing config file {:?}: {}", config_path, e);
                        eprintln!("Using default configuration");
                        Self::default()
                    }
                }
            }
            Err(_) => {
                // Config file doesn't exist, create default one
                let config = Self::default();
                if let Err(e) = config.save() {
                    eprintln!("Warning: Could not create default config file: {}", e);
                }
                config
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_file_path();
        
        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        
        Ok(())
    }

    fn config_file_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".dprs")
            .join("config")
    }

    pub fn get_key_binding(&self, mode: &str, key: &str) -> Option<&str> {
        let bindings = match mode {
            "normal" => &self.keybindings.normal_mode,
            "visual" => &self.keybindings.visual_mode,
            "command" => &self.keybindings.command_mode,
            _ => return None,
        };

        bindings.get(key).map(|s| s.as_str()).filter(|s| !s.is_empty())
    }

    pub fn should_auto_refresh(&self) -> bool {
        self.general.auto_refresh_interval > 0
    }

    pub fn auto_refresh_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.general.auto_refresh_interval)
    }

    pub fn get_color(&self, key: &str) -> Color {
        if let Some(hex) = self.colors.custom_colors.get(key) {
            Self::hex_to_color(hex).unwrap_or(Color::White)
        } else {
            Color::White
        }
    }

    fn hex_to_color(hex: &str) -> Option<Color> {
        if hex.len() == 7 && hex.starts_with('#') {
            let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
            let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
            let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        } else {
            None
        }
    }
}

pub fn key_event_to_string(key: crossterm::event::KeyEvent) -> String {
    let mut result = String::new();
    
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        result.push_str("Ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        result.push_str("Alt+");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        result.push_str("Shift+");
    }

    match key.code {
        KeyCode::Char(c) => result.push(c),
        KeyCode::Enter => result.push_str("Enter"),
        KeyCode::Esc => result.push_str("Escape"),
        KeyCode::Tab => result.push_str("Tab"),
        KeyCode::Backspace => result.push_str("Backspace"),
        KeyCode::Delete => result.push_str("Delete"),
        KeyCode::Insert => result.push_str("Insert"),
        KeyCode::Home => result.push_str("Home"),
        KeyCode::End => result.push_str("End"),
        KeyCode::PageUp => result.push_str("PageUp"),
        KeyCode::PageDown => result.push_str("PageDown"),
        KeyCode::Up => result.push_str("Up"),
        KeyCode::Down => result.push_str("Down"),
        KeyCode::Left => result.push_str("Left"),
        KeyCode::Right => result.push_str("Right"),
        KeyCode::F(n) => result.push_str(&format!("F{}", n)),
        _ => return "Unknown".to_string(),
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_color() {
        assert_eq!(Config::hex_to_color("#000000"), Some(Color::Rgb(0, 0, 0)));
        assert_eq!(Config::hex_to_color("#FfFfFF"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(Config::hex_to_color("#fF0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(Config::hex_to_color("invalid"), None);
        assert_eq!(Config::hex_to_color("#12345"), None); // Too short
        assert_eq!(Config::hex_to_color("#1234567"), None); // Too long
    }

    #[test]
    fn test_get_color_with_defaults() {
        let config = Config::default();
        // Should return black for background_main
        assert_eq!(config.get_color("background_main"), Color::Rgb(0, 0, 0));
        // Should return white for unknown color
        assert_eq!(config.get_color("unknown_color"), Color::White);
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
