use serde::{Deserialize, Deserializer};
use smithay::input::keyboard::ModifiersState;
use std::fs;

pub struct Modifiers {
    ctrl: bool,
    alt: bool,
    shift: bool,
    caps_lock: bool,
    logo: bool,
    num_lock: bool,
}

impl PartialEq<&ModifiersState> for Modifiers {
    fn eq(&self, other: &&ModifiersState) -> bool {
        !((self.ctrl && !other.ctrl)
            || (self.alt && !other.alt)
            || (self.shift && !other.shift)
            || (self.caps_lock && !other.caps_lock)
            || (self.logo && !other.logo)
            || (self.num_lock && !other.num_lock))
    }
}

#[allow(non_snake_case)]
fn deserialize_Modifiers<'de, D>(deserializer: D) -> Result<Modifiers, D::Error>
where
    D: Deserializer<'de>,
{
    let modifiers = Vec::<String>::deserialize(deserializer)?;

    Ok(Modifiers {
        ctrl: modifiers.contains(&"Ctrl".to_string()),
        alt: modifiers.contains(&"Alt".to_string()),
        shift: modifiers.contains(&"Shift".to_string()),
        caps_lock: modifiers.contains(&"CapsLock".to_string()),
        logo: modifiers.contains(&"Logo".to_string()),
        num_lock: modifiers.contains(&"NumLock".to_string()),
    })
}

#[derive(Deserialize)]
pub struct Config {
    pub border_thickness: i32,
    pub border_gap: i32,
    pub active_border: u32,
    pub inactive_border: u32,
    #[serde(deserialize_with = "deserialize_Modifiers")]
    pub workspace_modifier: Modifiers,
    #[serde(deserialize_with = "deserialize_Modifiers")]
    pub workspace_move_window_modifier: Modifiers,
}

pub fn get_config() -> Config {
    let home_path = std::env::var_os("HOME").expect("No HOME variable set.");

    let config_path = format!(
        "{}{}",
        home_path.to_string_lossy(),
        "/.config/mwm/config.toml"
    );
    let toml_string = fs::read_to_string(config_path)
        .expect("Unable to read config.toml file from ~/.config/mwm/config.toml");

    toml::from_str(&toml_string).expect("Unable to parse toml config.")
}
