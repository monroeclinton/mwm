use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
pub enum Actions {
    SelectLeftWindow,
    SelectRightWindow,
}

#[derive(Deserialize)]
pub struct Action {
    pub modifier: u16,
    pub keysym: u32,
    pub action: Actions,
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
    pub autostart: Vec<String>,
    pub actions: Vec<Action>,
    pub commands: Vec<Command>,
}

pub fn get_config() -> Config {
    let toml_string = fs::read_to_string("config.toml")
        .expect("Unable to read config.toml file.");

    toml::from_str(&toml_string)
        .expect("Unable to parse toml config.")
}
