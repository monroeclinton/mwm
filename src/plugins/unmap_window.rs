use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct UnmapWindow;

impl PluginHandler for UnmapWindow {
    fn on_unmap_notify(&mut self, ectx: EventContext<xcb::UnmapNotifyEvent>) -> Result<()> {
        let mut clients = ectx.clients.lock().unwrap();
        clients.hide(ectx.event.window());

        Ok(())
    }
}
