use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct DestroyWindow;

impl PluginHandler for DestroyWindow {
    fn on_destroy_notify(&mut self, ectx: EventContext<xcb::DestroyNotifyEvent>) -> Result<()> {
        let mut clients = ectx.clients.lock().unwrap();
        clients.destroy(ectx.event.window());

        Ok(())
    }
}
