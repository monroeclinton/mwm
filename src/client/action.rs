use crate::client::{Client, Clients};
use crate::config::Action;

impl Clients {
    pub fn handle_action(&mut self, _window: xcb::Window, action: Action) {
        // Handle close action
        if let (Action::Close, Some(window)) = (&action, self.active_window()) {
            let delete_window = xcb::intern_atom(&self.conn, false, "WM_DELETE_WINDOW")
                .get_reply()
                .unwrap();

            let reply =
                xcb_util::icccm::get_wm_protocols(&self.conn, window, self.conn.WM_PROTOCOLS())
                    .get_reply();

            let mut supports_wm_delete_window =
                reply.unwrap().atoms().contains(&delete_window.atom());

            if supports_wm_delete_window {
                let event = xcb::ClientMessageEvent::new(
                    32,
                    window,
                    self.conn.WM_PROTOCOLS(),
                    xcb::ClientMessageData::from_data32([
                        delete_window.atom(),
                        xcb::CURRENT_TIME,
                        0,
                        0,
                        0,
                    ]),
                );

                supports_wm_delete_window = xcb::send_event_checked(
                    &self.conn,
                    true,
                    window,
                    xcb::EVENT_MASK_NO_EVENT,
                    &event,
                )
                .request_check()
                .is_ok();
            }

            if !supports_wm_delete_window {
                xcb::set_close_down_mode(&self.conn, xcb::CLOSE_DOWN_DESTROY_ALL as u8);
                xcb::kill_client(&self.conn, window);
            }

            self.conn.flush();
        }

        let clients = self
            .clients
            .iter()
            .filter(|&c| c.visible && c.controlled)
            .cloned()
            .collect::<Vec<Client>>();

        let pos = clients
            .iter()
            .position(|c| Some(c.window) == self.active_window())
            .unwrap_or(0);

        // Handle the selection actions
        let new_pos = match action {
            Action::SelectAbove => {
                if clients.len() <= 1 {
                    0
                } else if pos == 0 && !clients.is_empty() {
                    clients.len() - 1
                } else {
                    pos - 1
                }
            }
            Action::SelectBelow => {
                if clients.len() <= 1 || pos >= clients.len() - 1 {
                    0
                } else {
                    pos + 1
                }
            }
            _ => pos,
        };

        let active_client = clients.get(new_pos);
        if let Some(client) = active_client {
            self.set_active_window(Some(client.window));
        }

        // Handle the window sizing actions
        let size = self
            .front_window_ratio
            .entry(self.active_workspace)
            .or_insert(0.5);

        match action {
            Action::ShrinkFront => {
                if *size > 0.10 {
                    *size -= 0.05;
                }
            }
            Action::ExpandFront => {
                if *size < 0.9 {
                    *size += 0.05;
                }
            }
            _ => (),
        };

        self.resize();
    }
}
