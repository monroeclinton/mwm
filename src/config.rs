use std::fs;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub enum Action {
    CloseWindow,
    SelectAboveWindow,
    SelectBelowWindow,
    ShrinkFrontWindow,
    ExpandFrontWindow,
}

#[derive(Deserialize)]
pub struct ActionKeyPress {
    pub modifier: u16,
    pub keysym: u32,
    pub action: Action,
}

#[derive(Deserialize)]
pub struct Command {
    pub modifier: u16,
    pub keysym: u32,
    pub command: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub border_thickness: u32,
    pub border_gap: u32,
    pub active_border: u32,
    pub inactive_border: u32,
    pub workspace_modifier: u16,
    pub workspace_move_window_modifier: u16,
    pub autostart: Vec<String>,
    pub actions: Vec<ActionKeyPress>,
    pub commands: Vec<Command>,
}

pub fn get_config() -> Config {
    let home_path = std::env::var_os("HOME")
        .expect("No HOME variable set.");

    let config_path = format!("{}{}", home_path.to_string_lossy(), "/.config/mwm/config.toml");
    let toml_string = fs::read_to_string(config_path)
        .expect("Unable to read config.toml file from ~/.config/mwm/config.toml");

    toml::from_str(&toml_string)
        .expect("Unable to parse toml config.")
}
