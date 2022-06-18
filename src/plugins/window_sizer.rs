use crate::config::Config;
use crate::client::{Clients, ResizeClients};
use crate::event::EventContext;
use std::sync::Arc;
use actix::{Actor, Context, Handler, Supervised, SystemService};

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

impl Actor for WindowSizer {
    type Context = Context<Self>;
}

impl Supervised for WindowSizer {}
impl SystemService for WindowSizer {}

impl Handler<EventContext<xcb::PropertyNotifyEvent>> for WindowSizer {
    type Result = ();

    fn handle(&mut self, ectx: EventContext<xcb::PropertyNotifyEvent>, _ctx: &mut Self::Context) -> Self::Result {
        if ectx.event.atom() == ectx.conn.WM_STRUT_PARTIAL() {
            self.resize_clients(ectx.conn, ectx.config);
        }
    }
}

impl Handler<EventContext<xcb::MapRequestEvent>> for WindowSizer {
    type Result = ();

    fn handle(&mut self, ectx: EventContext<xcb::MapRequestEvent>, _ctx: &mut Self::Context) -> Self::Result {
        self.resize_clients(ectx.conn, ectx.config);
    }
}

impl Handler<EventContext<xcb::UnmapNotifyEvent>> for WindowSizer {
    type Result = ();

    fn handle(&mut self, ectx: EventContext<xcb::UnmapNotifyEvent>, _ctx: &mut Self::Context) -> Self::Result {
        self.resize_clients(ectx.conn, ectx.config);
    }
}
