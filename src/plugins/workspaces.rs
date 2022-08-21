use crate::client::{SetActiveWorkspace, SetWindowWorkspace};
use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct Workspaces;

impl PluginHandler for Workspaces {
    fn on_client_message(&mut self, ectx: EventContext<xcb::ClientMessageEvent>) -> Result<()> {
        if ectx.event.type_() == ectx.conn.CURRENT_DESKTOP() {
            ectx.clients.do_send(SetActiveWorkspace {
                workspace: 1,
            });
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
            if ectx.config.workspace_modifier == ectx.event.state() {
                ectx.clients.do_send(SetActiveWorkspace {
                    workspace,
                });
            }

            if ectx.config.workspace_move_window_modifier == ectx.event.state() {
                ectx.clients.do_send(SetWindowWorkspace {
                    window: ectx.event.child(),
                    workspace: Some(workspace),
                });
            }
        }

        Ok(())
    }
}
