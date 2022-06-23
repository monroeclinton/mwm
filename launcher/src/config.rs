use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub width: u16,
    pub border_thickness: u16,
    pub border_color: u32,
    pub font_size: u32,
    pub font_family: String,
    pub font_color: u32,
    pub font_active_color: u32,
    pub background_color: u32,
    pub background_active_color: u32,
    pub close_keysym: u32,
    pub up_keysym: u32,
    pub down_keysym: u32,
}

pub fn get_config() -> Config {
    let toml_string = fs::read_to_string("launcher/config.toml")
        .expect("Unable to read config.toml file.");

    toml::from_str(&toml_string)
        .expect("Unable to parse toml config.")
}
