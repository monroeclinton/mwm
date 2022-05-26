use crate::plugin::MapRequestContext;
use actix::{Actor, Context, Handler, Supervised, SystemService};
use anyhow::Result;
use crate::window_manager::{CreateClient, WindowManager};

#[derive(Default)]
pub struct MapWindow;

impl Actor for MapWindow {
    type Context = Context<Self>;
}

impl Supervised for MapWindow {}
impl SystemService for MapWindow {}

impl Handler<MapRequestContext> for MapWindow {
    type Result = Result<()>;

    fn handle(&mut self, ectx: MapRequestContext, _ctx: &mut Context<Self>) -> Self::Result {
        if has_override_redirect(&ectx.conn, ectx.event.window) {
            return Ok(());
        }

        let values = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_ENTER_WINDOW)];

        xcb::change_window_attributes(&ectx.conn, ectx.event.window, &values);

        xcb::map_window(&ectx.conn, ectx.event.window);

        WindowManager::from_registry().do_send(CreateClient {
            window: ectx.event.window,
        });

        Ok(())
    }
}

fn has_override_redirect(conn: &xcb::Connection, window: xcb::Window) -> bool {
    let cookie = xcb::get_window_attributes(conn, window);

    if let Ok(attrs) = cookie.get_reply() {
        attrs.override_redirect()
    } else {
        false
    }
}
