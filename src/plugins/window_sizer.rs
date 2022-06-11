use crate::config::Config;
use crate::client::{Client, Clients, GetClients};
use crate::event::EventContext;
use crate::screen::get_screen;
use std::sync::Arc;
use actix::{
    Actor, ActorFutureExt, Context, Handler, ResponseActFuture,
    Supervised, SystemService, WrapFuture
};
use anyhow::Result;

#[derive(Default)]
pub struct WindowSizer {
    padding_top: u32,
    // TODO: Add rest of padding
}

impl WindowSizer {
    fn resize_clients(&mut self, conn: Arc<xcb_util::ewmh::Connection>, config: Arc<Config>) -> ResponseActFuture<Self, Result<()>> {
        let padding_top = self.padding_top;

        Clients::from_registry()
            .send(GetClients)
            .into_actor(self)
            .map(move |result, _actor, _ctx| {
                let clients = result?;

                let screen = get_screen(&conn);

                resize(
                    &conn,
                    clients,
                    screen.width_in_pixels() as usize,
                    screen.height_in_pixels() as usize,
                    config.border_thickness as usize,
                    config.border_gap as usize,
                    padding_top as usize,
                );

                Ok(())
            })
            .boxed_local()
    }
}

impl Actor for WindowSizer {
    type Context = Context<Self>;
}

impl Supervised for WindowSizer {}
impl SystemService for WindowSizer {}

impl Handler<EventContext<xcb::PropertyNotifyEvent>> for WindowSizer {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::PropertyNotifyEvent>, _ctx: &mut Self::Context) -> Self::Result {
        if ectx.event.atom() == ectx.conn.WM_STRUT_PARTIAL() {
            let cookie = xcb_util::ewmh::get_wm_strut_partial(&ectx.conn, ectx.event.window())
                .get_reply();

            if let Ok(struct_partial) = cookie {
                self.padding_top = self.padding_top + struct_partial.top;
            }
        }

        self.resize_clients(ectx.conn, ectx.config)
    }
}

impl Handler<EventContext<xcb::MapRequestEvent>> for WindowSizer {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::MapRequestEvent>, _ctx: &mut Self::Context) -> Self::Result {
        self.resize_clients(ectx.conn, ectx.config)
    }
}

impl Handler<EventContext<xcb::UnmapNotifyEvent>> for WindowSizer {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::UnmapNotifyEvent>, _ctx: &mut Self::Context) -> Self::Result {
        self.resize_clients(ectx.conn, ectx.config)
    }
}

fn resize(
    conn: &xcb_util::ewmh::Connection,
    clients: Vec<Client>,
    screen_width: usize,
    screen_height: usize,
    border_thickness: usize,
    border_gap: usize,
    padding_top: usize,
) {
    let border = border_thickness;
    let border_double = border * 2;
    let gap = border_gap;
    let gap_double = gap * 2;
    let available_height = screen_height - padding_top;

    let visible_clients = clients
        .iter()
        .filter(|&c| c.visible && c.controlled)
        .cloned()
        .collect::<Vec<Client>>();

    let clients_length = visible_clients.len();

    for (i, client) in visible_clients.iter().enumerate() {
        let (mut x, mut y) = (gap, gap + padding_top);

        let (mut width, mut height) = (
            screen_width - border_double - gap_double,
            available_height - border_double - gap_double,
        );

        if clients_length > 1 {
            width = (width - border_double - gap_double) / 2;

            if i > 0 {
                let window_height = (available_height) / (clients_length - 1);

                x = x + width + border_double + gap_double;
                y = y + window_height * (i - 1);

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
