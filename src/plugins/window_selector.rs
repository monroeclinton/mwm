use crate::client::{SetActiveWindow, HandleWindowAction};
use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct WindowSelector;

impl PluginHandler for WindowSelector {
    fn on_client_message(&mut self, ectx: EventContext<xcb::ClientMessageEvent>) -> Result<()> {
        ectx.clients.do_send(SetActiveWindow {
            config: ectx.config,
            window: Some(ectx.event.window()),
        });

        Ok(())
    }

    fn on_key_press(&mut self, ectx: EventContext<xcb::KeyPressEvent>) -> Result<()> {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);

        for action_key_press in ectx.config.actions.iter() {
            let keycode = key_symbols
                .get_keycode(action_key_press.keysym)
                .next()
                .expect("Unknown keycode found in window_selector plugin.");

            if keycode == ectx.event.detail() && action_key_press.modifier == ectx.event.state() {
                ectx.clients.do_send(HandleWindowAction {
                    config: ectx.config.clone(),
                    action: action_key_press.action.clone(),
                    window: ectx.event.event(),
                });
            }
        }

        Ok(())
    }

    fn on_enter_notify(&mut self, ectx: EventContext<xcb::EnterNotifyEvent>) -> Result<()> {
        ectx.clients.do_send(SetActiveWindow {
            config: ectx.config,
            window: Some(ectx.event.event()),
        });

        Ok(())
    }
}
