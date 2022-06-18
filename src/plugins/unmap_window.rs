use crate::client::{Clients, HideWindow};
use crate::event::EventContext;
use crate::plugin::PluginHandler;
use actix::SystemService;
use anyhow::Result;

#[derive(Default)]
pub struct UnmapWindow;

impl PluginHandler for UnmapWindow {
    fn on_unmap_notify(&mut self, ectx: EventContext<xcb::UnmapNotifyEvent>) -> Result<()> {
        Clients::from_registry().do_send(HideWindow {
            conn: ectx.conn,
            window: ectx.event.window(),
        });

        Ok(())
    }
}
