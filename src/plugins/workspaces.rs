use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct Workspaces;

impl PluginHandler for Workspaces {
    fn on_client_message(&mut self, ectx: EventContext<xcb::ClientMessageEvent>) -> Result<()> {
        if ectx.event.type_() == ectx.conn.CURRENT_DESKTOP() {
            let mut clients = ectx.clients.lock().unwrap();
            clients.set_active_workspace(1);
        }

        Ok(())
    }

    fn on_key_press(&mut self, ectx: EventContext<xcb::KeyPressEvent>) -> Result<()> {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);

        let mut active_workspace: Option<u8> = None;
        for workspace in 1..=9 {
            let keycode = key_symbols
                .get_keycode(x11::keysym::XK_0 + workspace as u32)
                .next()
                .expect("Unknown keycode found in workspaces plugin.");

            if keycode == ectx.event.detail() {
                active_workspace = Some(workspace);
                break;
            }
        }

        drop(key_symbols);

        if let Some(workspace) = active_workspace {
            let mut clients = ectx.clients.lock().unwrap();

            if ectx.config.workspace_modifier == ectx.event.state() {
                clients.set_active_workspace(workspace);
            }

            if ectx.config.workspace_move_window_modifier == ectx.event.state() {
                clients.set_window_workspace(ectx.event.child(), Some(workspace));
            }
        }

        Ok(())
    }
}
