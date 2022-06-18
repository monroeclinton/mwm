use crate::config::Config;
use crate::client::{Clients, ResizeClients};
use crate::event::EventContext;
use crate::plugin::PluginHandler;
use std::sync::Arc;
use actix::SystemService;
use anyhow::Result;

#[derive(Default)]
pub struct WindowSizer;

impl WindowSizer {
    fn resize_clients(&mut self, conn: Arc<xcb_util::ewmh::Connection>, config: Arc<Config>) {
        Clients::from_registry().do_send(ResizeClients {
            conn,
            config,
        });
    }
}

impl PluginHandler for WindowSizer {
    fn on_property_notify(&mut self, ectx: EventContext<xcb::PropertyNotifyEvent>) -> Result<()> {
        if ectx.event.atom() == ectx.conn.WM_STRUT_PARTIAL() {
            self.resize_clients(ectx.conn, ectx.config);
        }

        Ok(())
    }

    fn on_map_request(&mut self, ectx: EventContext<xcb::MapRequestEvent>) -> Result<()> {
        self.resize_clients(ectx.conn, ectx.config);

        Ok(())
    }

    fn on_unmap_notify(&mut self, ectx: EventContext<xcb::UnmapNotifyEvent>) -> Result<()> {
        self.resize_clients(ectx.conn, ectx.config);

        Ok(())
    }
}
