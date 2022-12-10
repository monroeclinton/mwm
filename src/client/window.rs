use crate::client::Clients;

impl Clients {
    pub fn destroy(&mut self, window: xcb::Window) {
        tracing::debug!("destroying client; window={}", window);

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
        tracing::debug!("hiding client; window={}", window);

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

        self.set_workspace_names();
        self.resize();
    }

    pub fn show(&mut self, window: xcb::Window) {
        tracing::debug!("showing client; window={}", window);

        for mut client in self.clients.iter_mut() {
            if window == client.window {
                if !client.visible {
                    xcb::map_window(&self.conn, client.window);
                }

                client.visible = true;
                break;
            }
        }

        self.conn.flush();

        self.set_workspace_names();
        self.resize();
    }

    pub fn enable_event_mask(&self, window: xcb::Window) {
        tracing::debug!("enable event mask; window={}", window);

        xcb::change_window_attributes(
            &self.conn,
            window,
            &[(
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_PROPERTY_CHANGE
                    | xcb::EVENT_MASK_STRUCTURE_NOTIFY
                    | xcb::EVENT_MASK_ENTER_WINDOW,
            )],
        );
    }

    pub fn disable_event_mask(&self, window: xcb::Window) {
        tracing::debug!("disable event mask; window={}", window);

        xcb::change_window_attributes(
            &self.conn,
            window,
            &[(
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_PROPERTY_CHANGE | xcb::EVENT_MASK_STRUCTURE_NOTIFY,
            )],
        );
    }

    pub fn set_controlled_status(&mut self, window: xcb::Window, status: bool) {
        tracing::debug!("set controlled status; window={}", window);

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
            tracing::debug!("set active status; window={:?}", window);
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
            tracing::debug!(
                "set previous active window to inactive; previous_window={:?}",
                self.active_window
            );
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

                let data = if client.full_screen {
                    self.conn.WM_STATE_FULLSCREEN()
                } else {
                    0
                };

                xcb::change_property(
                    &self.conn,
                    xcb::PROP_MODE_REPLACE as u8,
                    window,
                    self.conn.WM_STATE(),
                    xcb::ATOM_ATOM,
                    32,
                    &[data],
                );

                self.resize();
                break;
            }
        }
    }
}
