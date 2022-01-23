mod client;
mod config;
mod errors;
mod key;
mod macros;
mod plugin;
mod plugins;
mod window_manager;

use key::KeyPair;
use plugin::PluginHandler;
use std::process::Command;
use window_manager::WindowManager;

fn main() {
    let config = crate::config::Config {
        border_thickness: 2,
        border_gap: 4,
        active_border: 0x3b7a82,
        inactive_border: 0x444444,
    };

    // Create key maps
    let commands = command_map!((
        KeyPair {
            modifiers: xcb::MOD_MASK_1 as u16,
            keysym: x11::keysym::XK_p,
        },
        Box::new(|| Command::new("st"))
    ));

    // Add plugins
    let plugins: Vec<Box<dyn PluginHandler>> = vec![
        // Used for setting window layout
        Box::new(plugins::load_window_sizer_plugin()),
    ];

    WindowManager::new(config, commands, plugins).run();
}
