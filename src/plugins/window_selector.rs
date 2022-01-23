use crate::client::Client;
use crate::config::Config;
use crate::key::KeyPair;
use crate::plugin::{EnterNotifyContext, InitContext, KeyPressContext, Plugin, PluginHandler};
use std::collections::{HashMap, VecDeque};

// TODO: This plugin needs some clean up...
pub fn load_window_selector_plugin(events: HashMap<KeyPair, Event>) -> Plugin<PluginContext> {
    Plugin {
        context: PluginContext {
            events,
            active_window: 0,
        },
    }
}

#[macro_export]
macro_rules! selector_map {
    ( $( ($x:expr, $y:expr) ),* ) => {
        {
            let mut keys = std::collections::HashMap::<$crate::key::KeyPair, $crate::plugins::window_selector::Event>::new();

            $(
                keys.insert($x, $y);
            )*

            keys
        }
    };
}

pub enum Event {
    Forward,
    Backward,
}

pub struct PluginContext {
    events: HashMap<KeyPair, Event>,
    active_window: xcb::Window,
}

impl PluginHandler for Plugin<PluginContext> {
    fn init(&self, ictx: InitContext) {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(ictx.conn);
        for pair in self.context.events.keys() {
            match key_symbols.get_keycode(pair.keysym).next() {
                Some(keycode) => {
                    xcb::grab_key(
                        ictx.conn,
                        false,
                        ictx.screen.root(),
                        pair.modifiers,
                        keycode,
                        xcb::GRAB_MODE_ASYNC as u8,
                        xcb::GRAB_MODE_ASYNC as u8,
                    );
                }
                _ => {
                    dbg!("Failed to find keycode for keysym: {}", pair.keysym);
                }
            }
        }
    }

    fn on_key_press(&mut self, ectx: KeyPressContext) {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(ectx.conn);
        for pair in self.context.events.keys() {
            if let Some(keycode) = key_symbols.get_keycode(pair.keysym).next() {
                if keycode == ectx.event.detail() && pair.modifiers == ectx.event.state() {
                    if let Some(event) = self.context.events.get(pair) {
                        self.context.active_window =
                            move_window(&self.context.active_window, event, &ectx);
                    }
                }
            }
        }
    }

    fn on_enter_notify(&mut self, ectx: EnterNotifyContext) {
        self.context.active_window = ectx.event.event();
        set_active_window(ectx.conn, ectx.config, ectx.clients, ectx.event.event());
    }
}

fn set_active_window(
    conn: &xcb::Connection,
    config: &Config,
    clients: &VecDeque<Client>,
    window: xcb::Window,
) {
    let active_border = config.active_border;
    let inactive_border = config.inactive_border;

    xcb::set_input_focus(
        conn,
        xcb::INPUT_FOCUS_PARENT as u8,
        window,
        xcb::CURRENT_TIME,
    );

    xcb::change_window_attributes(conn, window, &[(xcb::CW_BORDER_PIXEL, active_border)]);

    for client in clients.iter() {
        if client.window != window {
            xcb::change_window_attributes(
                conn,
                client.window,
                &[(xcb::CW_BORDER_PIXEL, inactive_border)],
            );
        }
    }
}

fn move_window(active_window: &xcb::Window, event: &Event, ectx: &KeyPressContext) -> u32 {
    if let Some(pos) = ectx.clients.iter().position(|c| &c.window == active_window) {
        match event {
            Event::Forward => {
                let new_window_pos = if pos >= ectx.clients.len() - 1 {
                    0
                } else {
                    pos + 1
                };

                let window = ectx.clients.get(new_window_pos).unwrap().window;
                set_active_window(ectx.conn, ectx.config, ectx.clients, window);

                window
            }
            Event::Backward => {
                let new_window_pos = if pos == 0 && ectx.clients.len() == 0 {
                    0
                } else if pos == 0 && ectx.clients.len() > 0 {
                    ectx.clients.len() - 1
                } else {
                    pos - 1
                };

                let window = ectx.clients.get(new_window_pos).unwrap().window;
                set_active_window(ectx.conn, ectx.config, ectx.clients, window);

                window
            }
        }
    } else {
        0
    }
}
