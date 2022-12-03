use crate::client::{Client, Clients};

impl Clients {
    pub fn create(&mut self, window: xcb::Window) {
        let already_created = self.clients.iter().any(|c| c.window == window);

        if already_created {
            tracing::debug!("client already created");
            return;
        }

        self.enable_event_mask(window);

        let reply = xcb_util::ewmh::get_wm_window_type(&self.conn, window).get_reply();

        let mut controlled = true;

        if let Ok(window_type) = reply {
            let atoms = window_type.atoms();
            for atom in atoms {
                if *atom == self.conn.WM_WINDOW_TYPE_DOCK() {
                    self.dock_window = Some(window);
                    controlled = false;
                }

                if *atom == self.conn.WM_WINDOW_TYPE_DIALOG() {
                    controlled = false;
                }
            }
        }

        let cookie = xcb_util::ewmh::get_wm_strut_partial(&self.conn, window).get_reply();

        // TODO: Add other paddings
        let padding_top = if let Ok(struct_partial) = cookie {
            struct_partial.top
        } else {
            0
        };

        let workspace = if controlled {
            Some(self.active_workspace)
        } else {
            None
        };

        self.clients.push_front(Client {
            window,
            workspace,
            visible: true,
            controlled,
            full_screen: false,
            padding_top,
        });

        // Ensure border width and color is set for non-dock windows
        if self.dock_window != Some(window) {
            xcb::configure_window(
                &self.conn,
                window,
                &[(
                    xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
                    self.config.border_thickness,
                )],
            );

            xcb::change_window_attributes(
                &self.conn,
                window,
                &[(xcb::CW_BORDER_PIXEL, self.config.inactive_border)],
            );

            // Set window as active
            self.set_active_window(Some(window));
        }

        // Make sure window does not overlap with statusbar
        if controlled {
            xcb::configure_window(
                &self.conn,
                window,
                &[(xcb::CONFIG_WINDOW_Y as u16, self.get_padding_top() as u32)],
            );
        }

        xcb::map_window(&self.conn, window);

        self.conn.flush();

        self.refresh_clients();

        tracing::debug!("client created");
    }
}
