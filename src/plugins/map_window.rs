use crate::client::CreateClient;
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

        xcb::change_window_attributes(&ectx.conn, ectx.event.window(), &[
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_PROPERTY_CHANGE | xcb::EVENT_MASK_STRUCTURE_NOTIFY | xcb::EVENT_MASK_ENTER_WINDOW)
        ]);

        ectx.clients.do_send(CreateClient {
            window: ectx.event.window(),
        });

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
