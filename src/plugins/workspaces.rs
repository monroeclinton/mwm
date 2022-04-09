use crate::client::Client;
use crate::key::{KeyPair, grab_key};
use crate::plugin::{ConfigureRequestContext, EnterNotifyContext, InitContext, KeyPressContext, MapRequestContext, Plugin, PluginHandler, UnmapNotifyContext};
use std::collections::{HashMap, VecDeque};
use std::vec::Vec;
use anyhow::{Context, Result};

pub fn load_workspaces_plugin(count: u8) -> Plugin<PluginContext> {
    if count < 1 || count > 9 {
        panic!("The amount of workspaces must be between 1 and 9.");
    }

    let mut events = HashMap::new();
    let mut workspaces = HashMap::new();

    let modifiers = xcb::MOD_MASK_1 as u16;
    for w in 1..count {
        let keysym = x11::keysym::XK_0 + w as u32;
        events.insert(KeyPair { modifiers, keysym }, w);
        workspaces.insert(w, Vec::new());
    }

    Plugin {
        context: PluginContext {
            events,
            active_workspace: 1,
            workspaces,
        },
    }
}

pub struct PluginContext {
    events: HashMap<KeyPair, u8>,
    active_workspace: u8,
    workspaces: HashMap<u8, Vec<xcb::Window>>,
}

impl PluginHandler for Plugin<PluginContext> {
    fn init(&self, ictx: InitContext) {
        for pair in self.context.events.keys() {
            grab_key(pair, ictx.conn, ictx.screen.root());
        }
    }

    fn on_key_press(&mut self, ectx: KeyPressContext) -> Result<()> {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(ectx.conn);

        for (pair, workspace) in self.context.events.iter() {
            let keycode = key_symbols
                .get_keycode(pair.keysym)
                .next()
                .context("Unknown keycode found in workspaces plugin.")?;

            if keycode == ectx.event.detail() && pair.modifiers == ectx.event.state() {
                self.context.active_workspace = workspace.to_owned();
                unmap_windows(ectx.conn, ectx.clients);

                let active_windows = self.context.workspaces
                    .get(&self.context.active_workspace)
                    .context("Unable to find workspace.")?;

                map_windows(ectx.conn, active_windows);
            }
        }

        Ok(())
    }

    fn on_map_request(&mut self, ectx: MapRequestContext) -> Result<()> {
        let active_windows = self.context.workspaces
            .get_mut(&self.context.active_workspace)
            .context("Unable to find workspace.")?;

        active_windows.push(ectx.event.window());

        Ok(())
    }
}

fn unmap_windows(conn: &xcb::Connection, clients: &VecDeque<Client>) {
    for client in clients {
        xcb::unmap_window(conn, client.window);
    }
}

fn map_windows(conn: &xcb::Connection, windows: &Vec<xcb::Window>) {
    for window in windows {
        xcb::map_window(conn, window.to_owned());
    }
}
