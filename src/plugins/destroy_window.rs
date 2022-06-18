use crate::client::{Clients, DestroyClient};
use crate::event::EventContext;
use crate::plugin::PluginHandler;
use actix::SystemService;
use anyhow::Result;

#[derive(Default)]
pub struct DestroyWindow;

impl PluginHandler for DestroyWindow {
    fn on_destroy_notify(&mut self, ectx: EventContext<xcb::DestroyNotifyEvent>) -> Result<()> {
        Clients::from_registry().do_send(DestroyClient {
            conn: ectx.conn,
            window: ectx.event.window(),
        });

        Ok(())
    }
}
