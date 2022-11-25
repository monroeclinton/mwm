use crate::client::{SetControlledStatus, SetFullScreenStatus};
use crate::event::EventContext;
use crate::plugin::PluginHandler;
use crate::screen::get_screen;

#[derive(Default)]
pub struct ConfigureWindow;

impl PluginHandler for ConfigureWindow {
    fn on_client_message(
        &mut self,
        ectx: EventContext<xcb::ClientMessageEvent>,
    ) -> anyhow::Result<()> {
        if ectx.event.type_() == ectx.conn.WM_STATE() {
            let data = ectx.event.data().data32();

            let is_full_screen = if data[0] == xcb_util::ewmh::STATE_ADD {
                Some(true)
            } else {
                None
            };

            let toggle = data[1] == xcb_util::ewmh::STATE_TOGGLE;

            ectx.clients.do_send(SetFullScreenStatus {
                window: ectx.event.window(),
                status: is_full_screen,
                toggle,
            });
        }

        Ok(())
    }

    fn on_configure_request(
        &mut self,
        ectx: EventContext<xcb::ConfigureRequestEvent>,
    ) -> anyhow::Result<()> {
        let geomtry = xcb::get_geometry(&ectx.conn, ectx.event.window())
            .get_reply()
            .unwrap();

        let (mut width, mut height) = (geomtry.width(), geomtry.height());

        let mut values = Vec::new();

        if ectx.event.value_mask() & xcb::CONFIG_WINDOW_WIDTH as u16 > 0 {
            values.push((xcb::CONFIG_WINDOW_WIDTH as u16, ectx.event.width() as u32));
            width = ectx.event.width();
        }

        if ectx.event.value_mask() & xcb::CONFIG_WINDOW_HEIGHT as u16 > 0 {
            values.push((xcb::CONFIG_WINDOW_HEIGHT as u16, ectx.event.height() as u32));
            height = ectx.event.height();
        }

        if ectx.event.value_mask() & xcb::CONFIG_WINDOW_BORDER_WIDTH as u16 > 0 {
            values.push((
                xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
                ectx.event.border_width() as u32,
            ));
        }

        if ectx.event.value_mask() & xcb::CONFIG_WINDOW_SIBLING as u16 > 0 {
            values.push((
                xcb::CONFIG_WINDOW_SIBLING as u16,
                ectx.event.sibling() as u32,
            ));
        }

        if ectx.event.value_mask() & xcb::CONFIG_WINDOW_STACK_MODE as u16 > 0 {
            values.push((
                xcb::CONFIG_WINDOW_STACK_MODE as u16,
                ectx.event.stack_mode() as u32,
            ));
        }

        // Override dialog configure requests to center window
        let reply = xcb::get_property(
            &ectx.conn,
            false,
            ectx.event.window(),
            xcb::ATOM_WM_TRANSIENT_FOR,
            xcb::ATOM_WINDOW,
            0,
            1,
        )
        .get_reply();

        let is_transient = match reply {
            Ok(property) => {
                property.format() == 32
                    || property.type_() == xcb::ATOM_WINDOW
                    || property.value_len() != 0
            }
            _ => false,
        };

        let is_dialog = xcb_util::ewmh::get_wm_window_type(&ectx.conn, ectx.event.window())
            .get_reply()
            .map_or(false, |window_type| {
                window_type
                    .atoms()
                    .contains(&ectx.conn.WM_WINDOW_TYPE_DIALOG())
            });

        // Override coordinates for dialog windows to center it
        if is_transient || is_dialog {
            let screen = get_screen(&ectx.conn);
            let x = (screen.width_in_pixels() - width) / 2;
            let y = (screen.height_in_pixels() - height) / 2;

            values.push((xcb::CONFIG_WINDOW_X as u16, x as u32));
            values.push((xcb::CONFIG_WINDOW_Y as u16, y as u32));
        } else {
            if ectx.event.value_mask() & xcb::CONFIG_WINDOW_X as u16 > 0 {
                values.push((xcb::CONFIG_WINDOW_X as u16, ectx.event.x() as u32));
            }

            if ectx.event.value_mask() & xcb::CONFIG_WINDOW_Y as u16 > 0 {
                values.push((xcb::CONFIG_WINDOW_Y as u16, ectx.event.y() as u32));
            }
        }

        xcb::configure_window(&ectx.conn, ectx.event.window(), &values);

        ectx.conn.flush();

        Ok(())
    }

    fn on_property_notify(
        &mut self,
        ectx: EventContext<xcb::PropertyNotifyEvent>,
    ) -> anyhow::Result<()> {
        if ectx.event.atom() == ectx.conn.WM_WINDOW_TYPE() {
            let reply =
                xcb_util::ewmh::get_wm_window_type(&ectx.conn, ectx.event.window()).get_reply();

            if let Ok(reply) = reply {
                let atoms = reply.atoms();

                for atom in atoms {
                    if *atom == ectx.conn.WM_WINDOW_TYPE_DOCK() {
                        ectx.clients.do_send(SetControlledStatus {
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
