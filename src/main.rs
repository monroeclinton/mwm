mod client;
mod errors;
mod key;
mod window_manager;

use key::{KeyPair};
use window_manager::{WindowManager, Handler, Event};
use std::collections::{HashMap};
use std::process::Command;

fn main() {
    let mut keys = HashMap::new();

    keys.insert(
        KeyPair {
            modifiers: xcb::MOD_MASK_1 as u16,
            keysym: x11::keysym::XK_j, 
        },
        Handler {
            command: None,
            event: Some(Event::Forward),
        }
    );

    keys.insert(
        KeyPair {
            modifiers: xcb::MOD_MASK_1 as u16,
            keysym: x11::keysym::XK_p, 
        },
        Handler {
            command: Some(Box::new(|| Command::new("st"))),
            event: None,
        }
    );

    let mut wm = WindowManager::new(keys);
    wm.run();
}
