use crate::client::Client;
use crate::plugin::{MapRequestContext, Plugin, PluginHandler};
use std::collections::VecDeque;

pub fn load_window_sizer_plugin() -> Plugin<PluginContext> {
    Plugin {
        context: PluginContext {},
    }
}

pub struct PluginContext;

impl PluginHandler for Plugin<PluginContext> {
    fn on_map_request(&mut self, ectx: MapRequestContext) {
        resize(
            ectx.conn,
            ectx.clients,
            ectx.screen.width_in_pixels() as usize,
            ectx.screen.height_in_pixels() as usize,
            ectx.config.border_thickness,
            ectx.config.border_gap,
        );
    }
}

fn resize(
    conn: &xcb::Connection,
    clients: &VecDeque<Client>,
    screen_width: usize,
    screen_height: usize,
    border_thickness: u32,
    border_gap: u32,
) {
    let border = border_thickness as usize;
    let border_double = border * 2;
    let gap = border_gap as usize;
    let gap_double = gap * 2;
    let clients_length = clients.len();

    for (i, client) in clients.iter().enumerate() {
        let (mut x, mut y) = (gap, gap);

        let (mut width, mut height) = (
            screen_width - border_double - gap_double,
            screen_height - border_double - gap_double,
        );

        if clients_length > 1 {
            width = (width - border_double - gap_double) / 2;

            if i > 0 {
                let window_height = screen_height / (clients_length - 1);

                x = width + border_double + gap_double + gap;
                y = window_height * (i - 1) + gap;

                height = window_height - border_double - gap_double;
            }
        }

        xcb::configure_window(
            conn,
            client.window,
            &[
                (xcb::CONFIG_WINDOW_X as u16, x as u32),
                (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                (xcb::CONFIG_WINDOW_WIDTH as u16, width as u32),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, height as u32),
                (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border as u32),
            ],
        );
    }
}
