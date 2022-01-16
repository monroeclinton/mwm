mod client;
mod config;
mod errors;
mod key;
mod macros;
mod plugin;
mod plugins;
mod window_manager;

use key::KeyPair;
use window_manager::{WindowManager};
use std::process::Command;

fn main() {
    let config = crate::config::Config {
        // Create key maps
        commands: key_map!(
            (
                KeyPair {
                    modifiers: xcb::MOD_MASK_1 as u16,
                    keysym: x11::keysym::XK_p,
                },
                Box::new(|| Command::new("st"))
            )
        ),
        // Add plugins
        plugins: vec![
            Box::new(plugins::load_window_mapper_plugin()),
        ],
        // Border colors
        border_thickness: 2,
        border_gap: 4,
        active_border: 0x3b7a82,
        inactive_border: 0x444444,
    };

    WindowManager::new(config).run();
}
