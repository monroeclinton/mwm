use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub height: u16,
    pub margin: u16,
    pub font_size: u32,
    pub font_family: String,
    pub font_color: u32,
    pub font_active_color: u32,
    pub background_color: u32,
    pub background_active_color: u32,
    pub workspace_width: u32,
}

pub fn get_config() -> Config {
    let home_path = std::env::var_os("HOME").expect("No HOME variable set.");

    let config_path = format!(
        "{}{}",
        home_path.to_string_lossy(),
        "/.config/mwm/statusbar.toml"
    );
    let toml_string = fs::read_to_string(config_path)
        .expect("Unable to read config.toml file from ~/.config/mwm/statusbar.toml");

    toml::from_str(&toml_string).expect("Unable to parse toml config.")
}
