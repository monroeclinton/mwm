use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct WindowSizer;

impl PluginHandler for WindowSizer {
    fn on_property_notify(&mut self, ectx: EventContext<xcb::PropertyNotifyEvent>) -> Result<()> {
        if ectx.event.atom() == ectx.conn.WM_STRUT_PARTIAL() {
            let mut clients = ectx.clients.lock().unwrap();
            clients.resize();
        }

        Ok(())
    }

    fn on_map_request(&mut self, ectx: EventContext<xcb::MapRequestEvent>) -> Result<()> {
        let mut clients = ectx.clients.lock().unwrap();
        clients.resize();

        Ok(())
    }

    fn on_unmap_notify(&mut self, ectx: EventContext<xcb::UnmapNotifyEvent>) -> Result<()> {
        let mut clients = ectx.clients.lock().unwrap();
        clients.resize();

        Ok(())
    }
}
