use crate::client::{Clients, HideWindow};
use crate::event::EventContext;
use actix::{Actor, ActorFutureExt, Context, Handler, ResponseActFuture, Supervised, SystemService, WrapFuture};
use anyhow::Result;

#[derive(Default)]
pub struct UnmapWindow;

impl Actor for UnmapWindow {
    type Context = Context<Self>;
}

impl Supervised for UnmapWindow {}
impl SystemService for UnmapWindow {}

impl Handler<EventContext<xcb::UnmapNotifyEvent>> for UnmapWindow {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::UnmapNotifyEvent>, _ctx: &mut Context<Self>) -> Self::Result {
        Clients::from_registry()
            .send(HideWindow {
                conn: ectx.conn,
                window: ectx.event.window(),
            })
            .into_actor(self)
            .map(|_, _, _| Ok(()))
            .boxed_local()
    }
}
