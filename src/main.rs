mod client;
mod config;
mod errors;
mod key;
mod macros;
mod window_manager;

use key::KeyPair;
use window_manager::{WindowManager, Handler, Event};
use std::process::Command;

fn main() {
    let config = crate::config::Config {
        keys: key_map!(
            (
                KeyPair {
                    modifiers: xcb::MOD_MASK_1 as u16,
                    keysym: x11::keysym::XK_j,
                },
                Handler {
                    command: None,
                    event: Some(Event::Forward),
                }
            ),
            (
                KeyPair {
                    modifiers: xcb::MOD_MASK_1 as u16,
                    keysym: x11::keysym::XK_p,
                },
                Handler {
                    command: Some(Box::new(|| Command::new("st"))),
                    event: None,
                }
            )
        )
    };

    (WindowManager::new(config)).run();
}
