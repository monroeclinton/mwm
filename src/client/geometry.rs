use crate::client::{Client, Clients};
use crate::screen::get_screen;

impl Clients {
    pub fn get_padding_top(&self) -> usize {
        self.clients
            .iter()
            .filter(|&c| c.visible)
            .fold(0, |acc, c| acc + c.padding_top as usize)
    }

    pub fn resize(&mut self) {
        tracing::debug!("resizing clients");

        let screen = get_screen(&self.conn);

        let screen_width = screen.width_in_pixels() as usize;
        let screen_height = screen.height_in_pixels() as usize;
        let border = self.config.border_thickness as usize;
        let border_double = border * 2;
        let gap = self.config.border_gap as usize;
        let gap_double = gap * 2;

        let padding_top = self.get_padding_top();

        let max_clients = (screen_height - padding_top) / (gap_double + border_double) - 1;

        let visible_clients = self
            .clients
            .iter()
            .filter(|&c| c.visible && c.controlled)
            .take(max_clients)
            .cloned()
            .collect::<Vec<Client>>();

        let clients_length = visible_clients.len();
        let available_height = screen_height - padding_top;

        let front_window_ratio = *self
            .front_window_ratio
            .entry(self.active_workspace)
            .or_insert(0.5);

        // Tile windows
        for (i, client) in visible_clients.iter().enumerate() {
            let (mut x, mut y) = (gap, gap + padding_top);

            let (mut width, mut height) = (
                screen_width - border_double - gap_double,
                available_height - border_double - gap_double,
            );

            if clients_length > 1 {
                width -= border_double - gap_double;

                let front_window_width = (width as f32 * front_window_ratio) as usize;
                let window_height = (available_height) / (clients_length - 1);

                if i > 0 {
                    width -= front_window_width;
                    height = window_height - border_double - gap_double;

                    x += front_window_width + border_double + gap_double;
                    y += window_height * (i - 1);
                } else {
                    width = front_window_width;
                }
            }

            self.disable_event_mask(client.window);

            xcb::configure_window(
                &self.conn,
                client.window,
                &[
                    (xcb::CONFIG_WINDOW_X as u16, x as u32),
                    (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, width as u32),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, height as u32),
                    (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border as u32),
                ],
            );

            self.enable_event_mask(client.window);
        }

        // Full screen windows
        for client in visible_clients.iter().filter(|&c| c.full_screen) {
            xcb::configure_window(
                &self.conn,
                client.window,
                &[
                    (xcb::CONFIG_WINDOW_X as u16, 0),
                    (xcb::CONFIG_WINDOW_Y as u16, 0),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, screen_width as u32),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, screen_height as u32),
                    (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, 0),
                ],
            );
        }

        self.conn.flush();
    }
}
