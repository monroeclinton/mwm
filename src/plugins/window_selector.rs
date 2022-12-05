use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct WindowSelector;

impl PluginHandler for WindowSelector {
    fn on_key_press(&mut self, ectx: EventContext<xcb::KeyPressEvent>) -> Result<()> {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);

        for action_key_press in ectx.config.actions.iter() {
            let keycode = key_symbols
                .get_keycode(action_key_press.keysym)
                .next()
                .expect("Unknown keycode found in window_selector plugin.");

            if keycode == ectx.event.detail() && action_key_press.modifier == ectx.event.state() {
                let mut clients = ectx.clients.lock().unwrap();
                clients.handle_action(ectx.event.event(), action_key_press.action.clone());
            }
        }

        Ok(())
    }

    fn on_enter_notify(&mut self, ectx: EventContext<xcb::EnterNotifyEvent>) -> Result<()> {
        let mut clients = ectx.clients.lock().unwrap();
        clients.set_active_window(Some(ectx.event.event()));

        Ok(())
    }
}
