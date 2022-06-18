use crate::client::{Clients, HideWindow};
use crate::event::EventContext;
use actix::{Actor, Context, Handler, Supervised, SystemService};

#[derive(Default)]
pub struct UnmapWindow;

impl Actor for UnmapWindow {
    type Context = Context<Self>;
}

impl Supervised for UnmapWindow {}
impl SystemService for UnmapWindow {}

impl Handler<EventContext<xcb::UnmapNotifyEvent>> for UnmapWindow {
    type Result = ();

    fn handle(&mut self, ectx: EventContext<xcb::UnmapNotifyEvent>, _ctx: &mut Context<Self>) -> Self::Result {
        Clients::from_registry().do_send(HideWindow {
            conn: ectx.conn,
            window: ectx.event.window(),
        });
    }
}
