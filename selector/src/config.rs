use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub width: u16,
    pub border_thickness: u16,
    pub border_color: u32,
    pub font_size: u16,
    pub font_family: String,
    pub font_color: u32,
    pub font_active_color: u32,
    pub background_color: u32,
    pub background_active_color: u32,
    pub modifier: u16,
    pub close_keysym: u32,
    pub select_keysym: u32,
    pub up_keysym: u32,
    pub down_keysym: u32,
}

pub fn get_config() -> Config {
    let home_path = std::env::var_os("HOME").expect("No HOME variable set.");

    let config_path = format!(
        "{}{}",
        home_path.to_string_lossy(),
        "/.config/mwm/selector.toml"
    );
    let toml_string = fs::read_to_string(config_path)
        .expect("Unable to read config.toml file from ~/.config/mwm/selector.toml");

    toml::from_str(&toml_string).expect("Unable to parse toml config.")
}
