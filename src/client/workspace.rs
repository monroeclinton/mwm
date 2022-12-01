use crate::client::Clients;

impl Clients {
    pub fn set_active_workspace(&mut self, workspace: u8) {
        self.active_workspace = workspace;

        for mut client in self.clients.iter_mut().filter(|c| c.controlled) {
            if Some(self.active_workspace) == client.workspace {
                if !client.visible {
                    xcb::map_window(&self.conn, client.window);
                }

                client.visible = true;
            } else {
                if client.visible {
                    xcb::unmap_window(&self.conn, client.window);
                }

                client.visible = false;
            }
        }

        xcb_util::ewmh::set_current_desktop(&self.conn, 0, self.active_workspace as u32);

        self.set_active_window(None);

        self.resize();

        self.conn.flush();
    }

    pub fn set_window_workspace(&mut self, window: xcb::Window, workspace: Option<u8>) {
        if Some(self.active_workspace) == workspace {
            return;
        }

        for client in self.clients.iter_mut() {
            if client.window == window {
                client.workspace = workspace;

                self.hide(window);

                break;
            }
        }

        self.refresh_clients();
    }

    pub fn set_workspace_names(&mut self) {
        let names = (1..=9)
            .map(|i: u8| {
                let count = self
                    .clients
                    .iter()
                    .filter(|c| c.workspace == Some(i))
                    .count();

                let count_string = count
                    .to_string()
                    .replace('0', "⁰")
                    .replace('1', "¹")
                    .replace('2', "²")
                    .replace('3', "³")
                    .replace('4', "⁴")
                    .replace('5', "⁵")
                    .replace('6', "⁶")
                    .replace('7', "⁷")
                    .replace('8', "⁸")
                    .replace('9', "⁹");

                if count > 0 {
                    format!("{}{}", i, count_string)
                } else {
                    i.to_string()
                }
            })
            .collect::<Vec<String>>();

        xcb_util::ewmh::set_desktop_names(&self.conn, 0, names.iter().map(|s| s.as_ref()));
    }
}
