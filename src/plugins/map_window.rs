use crate::client::{Clients, CreateClient};
use crate::event::EventContext;
use actix::{Actor, Context, Handler, Supervised, SystemService};

#[derive(Default)]
pub struct MapWindow;

impl Actor for MapWindow {
    type Context = Context<Self>;
}

impl Supervised for MapWindow {}
impl SystemService for MapWindow {}

impl Handler<EventContext<xcb::MapRequestEvent>> for MapWindow {
    type Result = ();

    fn handle(&mut self, ectx: EventContext<xcb::MapRequestEvent>, _ctx: &mut Context<Self>) -> Self::Result {
        if has_override_redirect(&ectx.conn, ectx.event.window()) {
            return;
        }

        let values = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_PROPERTY_CHANGE | xcb::EVENT_MASK_STRUCTURE_NOTIFY | xcb::EVENT_MASK_ENTER_WINDOW)];

        xcb::change_window_attributes(&ectx.conn, ectx.event.window(), &values);

        xcb::map_window(&ectx.conn, ectx.event.window());

        ectx.conn.flush();

        Clients::from_registry().do_send(CreateClient {
            conn: ectx.conn,
            window: ectx.event.window(),
        });
    }
}

fn has_override_redirect(conn: &xcb_util::ewmh::Connection, window: xcb::Window) -> bool {
    let cookie = xcb::get_window_attributes(conn, window);

    if let Ok(attrs) = cookie.get_reply() {
        attrs.override_redirect()
    } else {
        false
    }
}
