use crate::client::DestroyClient;
use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct DestroyWindow;

impl PluginHandler for DestroyWindow {
    fn on_destroy_notify(&mut self, ectx: EventContext<xcb::DestroyNotifyEvent>) -> Result<()> {
        ectx.clients.do_send(DestroyClient {
            window: ectx.event.window(),
        });

        Ok(())
    }
}
