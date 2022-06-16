use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub height: u16,
    pub font_size: u32,
    pub font_family: String,
    pub font_color: u32,
    pub font_active_color: u32,
    pub background_color: u32,
    pub background_active_color: u32,
    pub workspace_padding: u32,
}

pub fn get_config() -> Config {
    let toml_string = fs::read_to_string("statusbar/config.toml")
        .expect("Unable to read config.toml file.");

    toml::from_str(&toml_string)
        .expect("Unable to parse toml config.")
}
