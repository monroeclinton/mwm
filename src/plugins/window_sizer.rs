use crate::client::ResizeClients;
use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct WindowSizer;

impl PluginHandler for WindowSizer {
    fn on_property_notify(&mut self, ectx: EventContext<xcb::PropertyNotifyEvent>) -> Result<()> {
        if ectx.event.atom() == ectx.conn.WM_STRUT_PARTIAL() {
            ectx.clients.do_send(ResizeClients {
                config: ectx.config,
            });
        }

        Ok(())
    }

    fn on_map_request(&mut self, ectx: EventContext<xcb::MapRequestEvent>) -> Result<()> {
        ectx.clients.do_send(ResizeClients {
            config: ectx.config,
        });

        Ok(())
    }

    fn on_unmap_notify(&mut self, ectx: EventContext<xcb::UnmapNotifyEvent>) -> Result<()> {
        ectx.clients.do_send(ResizeClients {
            config: ectx.config,
        });

        Ok(())
    }
}
