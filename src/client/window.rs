use crate::client::Clients;

impl Clients {
    pub fn destroy(&mut self, window: xcb::Window) {
        self.clients.retain(|c| c.window != window);

        if self.active_window == Some(window) {
            let active_window = self
                .clients
                .iter()
                .filter(|c| c.controlled)
                .next()
                .map_or(None, |c| Some(c.window));
            self.set_active_window(active_window);
        }

        self.refresh_clients();
    }

    pub fn hide(&mut self, window: xcb::Window) {
        for mut client in self.clients.iter_mut() {
            if window == client.window {
                if client.visible {
                    xcb::unmap_window(&self.conn, client.window);
                }

                client.visible = false;
                break;
            }
        }

        self.conn.flush();

        self.resize();
    }

    pub fn set_controlled_status(&mut self, window: xcb::Window, status: bool) {
        for mut client in self.clients.iter_mut() {
            if window == client.window {
                client.controlled = status;
                break;
            }
        }
    }

    pub fn set_active_window(&mut self, window: Option<xcb::Window>) {
        if window == self.dock_window {
            return;
        }

        let active_border = self.config.active_border;
        let inactive_border = self.config.inactive_border;

        if let Some(window) = window {
            xcb::set_input_focus(
                &self.conn,
                xcb::INPUT_FOCUS_PARENT as u8,
                window,
                xcb::CURRENT_TIME,
            );
            xcb::change_window_attributes(
                &self.conn,
                window,
                &[(xcb::CW_BORDER_PIXEL, active_border)],
            );
        }

        xcb_util::ewmh::set_active_window(&self.conn, 0, window.unwrap_or(xcb::WINDOW_NONE));

        if window != self.active_window {
            if let Some(active_window) = self.active_window {
                xcb::change_window_attributes(
                    &self.conn,
                    active_window,
                    &[(xcb::CW_BORDER_PIXEL, inactive_border)],
                );
            }

            self.active_window = window;
        }

        self.conn.flush();
    }

    pub fn set_full_screen(&mut self, window: xcb::Window, status: Option<bool>, toggle: bool) {
        for mut client in self.clients.iter_mut() {
            if window == client.window {
                client.full_screen = Some(true) == status || (!client.full_screen && toggle);
                self.resize();
                break;
            }
        }
    }
}
