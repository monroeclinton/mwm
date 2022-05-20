use crate::client::Client;
use crate::config::Config;
use crate::key::{KeyPair, grab_key};
use crate::plugin::{EnterNotifyContext, InitContext, KeyPressContext, PluginHandler};
use std::collections::{HashMap, VecDeque};
use anyhow::{Context, Result};

#[macro_export]
macro_rules! selector_map {
    ( $( ($x:expr, $y:expr) ),* ) => {
        {
            let mut keys = std::collections::HashMap::<
                $crate::key::KeyPair,
                $crate::plugins::window_selector::Event
            >::new();

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

pub struct WindowSelector {
    events: HashMap<KeyPair, Event>,
    active_window: xcb::Window,
}

impl WindowSelector {
    pub fn new(events: HashMap<KeyPair, Event>) -> Self {
        Self {
            events,
            active_window: 0,
        }
    }
}

impl PluginHandler for WindowSelector {
    fn init(&self, ictx: InitContext) {
        for pair in self.events.keys() {
            grab_key(pair, ictx.conn, ictx.screen.root());
        }
    }

    fn on_key_press(&mut self, ectx: KeyPressContext) -> Result<()> {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(ectx.conn);

        for (pair, event) in self.events.iter() {
            let keycode = key_symbols
                .get_keycode(pair.keysym)
                .next()
                .context("Unknown keycode found in window_selector plugin.")?;

            if keycode == ectx.event.detail() && pair.modifiers == ectx.event.state() {
                if let Some(window) = move_window(&self.active_window, event, &ectx)? {
                    self.active_window = window;
                }
            }
        }

        Ok(())
    }

    fn on_enter_notify(&mut self, ectx: EnterNotifyContext) -> Result<()> {
        self.active_window = ectx.event.event();
        set_active_window(ectx.conn, ectx.config, ectx.clients, ectx.event.event());
        Ok(())
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

fn move_window(
    active_window: &xcb::Window,
    event: &Event,
    ectx: &KeyPressContext
) -> Result<Option<xcb::Window>> {
    let pos = ectx.clients
        .iter()
        .position(|c| &c.window == active_window)
        .unwrap_or(0);

    let new_window_pos = match event {
        Event::Forward => {
            if pos >= ectx.clients.len() - 1 {
                0
            } else {
                pos + 1
            }
        },
        Event::Backward => {
            if pos == 0 && ectx.clients.len() == 0 {
                0
            } else if pos == 0 && ectx.clients.len() > 0 {
                ectx.clients.len() - 1
            } else {
                pos - 1
            }
        },
    };

    if let Some(client) = ectx.clients.get(new_window_pos) {
        let window = client.window;
        set_active_window(ectx.conn, ectx.config, ectx.clients, window);
        Ok(Some(window))
    } else {
        Ok(None)
    }
}
