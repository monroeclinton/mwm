use crate::client::{Clients, SetControlledStatus};
use crate::event::EventContext;
use crate::plugin::PluginHandler;
use actix::SystemService;

#[derive(Default)]
pub struct ConfigureWindow;

impl PluginHandler for ConfigureWindow {
    fn on_configure_request(&mut self, ectx: EventContext<xcb::ConfigureRequestEvent>) -> anyhow::Result<()> {
        let values = vec![
            (xcb::CONFIG_WINDOW_X as u16, ectx.event.x() as u32),
            (xcb::CONFIG_WINDOW_Y as u16, ectx.event.y() as u32),
            (xcb::CONFIG_WINDOW_WIDTH as u16, ectx.event.width() as u32),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, ectx.event.height() as u32),
            (
                xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
                ectx.event.border_width() as u32,
            ),
            (xcb::CONFIG_WINDOW_SIBLING as u16, ectx.event.sibling() as u32), // Default: NONE
            (
                xcb::CONFIG_WINDOW_STACK_MODE as u16,
                ectx.event.stack_mode() as u32,
            ), // Default: STACK_MODE_ABOVE
        ];

        xcb::configure_window(&ectx.conn, ectx.event.window(), &values);

        ectx.conn.flush();

        Ok(())
    }

    fn on_property_notify(&mut self, ectx: EventContext<xcb::PropertyNotifyEvent>) -> anyhow::Result<()> {
        if ectx.event.atom() == ectx.conn.WM_WINDOW_TYPE() {
            let reply = xcb_util::ewmh::get_wm_window_type(&ectx.conn, ectx.event.window())
                .get_reply();

            if let Ok(reply) = reply {
                let atoms = reply.atoms();

                for atom in atoms {
                    if *atom == ectx.conn.WM_WINDOW_TYPE_DOCK() {
                        Clients::from_registry().do_send(SetControlledStatus {
                            conn: ectx.conn.clone(),
                            window: ectx.event.window(),
                            status: false,
                        });
                    }
                }
            }
        }

        Ok(())
    }
}
