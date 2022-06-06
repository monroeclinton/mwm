use crate::client::{Clients, DestroyClient};
use crate::event::EventContext;
use actix::{Actor, Context, Handler, Supervised, SystemService};
use anyhow::Result;

#[derive(Default)]
pub struct DestroyWindow;

impl Actor for DestroyWindow {
    type Context = Context<Self>;
}

impl Supervised for DestroyWindow {}
impl SystemService for DestroyWindow {}

impl Handler<EventContext<xcb::DestroyNotifyEvent>> for DestroyWindow {
    type Result = Result<()>;

    fn handle(&mut self, ectx: EventContext<xcb::DestroyNotifyEvent>, _ctx: &mut Context<Self>) -> Self::Result {
        Clients::from_registry().do_send(DestroyClient {
            conn: ectx.conn,
            window: ectx.event.window(),
        });

        Ok(())
    }
}
