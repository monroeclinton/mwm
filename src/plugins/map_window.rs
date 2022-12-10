use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct MapWindow;

impl PluginHandler for MapWindow {
    fn on_map_request(&mut self, ectx: EventContext<xcb::MapRequestEvent>) -> Result<()> {
        if has_override_redirect(&ectx.conn, ectx.event.window()) {
            return Ok(());
        }

        let mut clients = ectx.clients.lock().unwrap();
        clients.create(ectx.event.window());
        clients.show(ectx.event.window());

        Ok(())
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
