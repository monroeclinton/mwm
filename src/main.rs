mod client;
mod config;
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
        // Handle movement between windows
        Box::new(plugins::WindowSelector::new(selector_map!(
            (
                KeyPair {
                    modifiers: xcb::MOD_MASK_1 as u16,
                    keysym: x11::keysym::XK_j,
                },
                plugins::window_selector::Event::Forward
            ),
            (
                KeyPair {
                    modifiers: xcb::MOD_MASK_1 as u16,
                    keysym: x11::keysym::XK_k,
                },
                plugins::window_selector::Event::Backward
            )
        ))),
        // Used for setting window layout
        Box::new(plugins::WindowSizer::new()),
        // Workspaces
        Box::new(plugins::Workspaces::new(9)),
    ];

    WindowManager::new(config, commands, plugins).run();
}
