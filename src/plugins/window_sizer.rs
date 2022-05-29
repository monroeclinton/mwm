use crate::client::{Client, Clients, GetClients};
use crate::event::{EventContext, MapRequestEvent};
use actix::{Actor, ActorFutureExt, Context, Handler, ResponseActFuture, Supervised, SystemService, WrapFuture};
use anyhow::Result;

#[derive(Default)]
pub struct WindowSizer;

impl Actor for WindowSizer {
    type Context = Context<Self>;
}

impl Supervised for WindowSizer {}
impl SystemService for WindowSizer {}

impl Handler<EventContext<MapRequestEvent>> for WindowSizer {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<MapRequestEvent>, _ctx: &mut Self::Context) -> Self::Result {
        Clients::from_registry()
            .send(GetClients)
            .into_actor(self)
            .map(move |result, _actor, _ctx| {
                let clients = result?;

                let screen = ectx.conn.get_setup().roots().next()
                    .expect("Unable to find a screen.");

                resize(
                    &ectx.conn,
                    &clients,
                    screen.width_in_pixels() as usize,
                    screen.height_in_pixels() as usize,
                    ectx.config.border_thickness,
                    ectx.config.border_gap,
                );

                Ok(())
            })
            .boxed_local()
    }
}

fn resize(
    conn: &xcb::Connection,
    clients: &[Client],
    screen_width: usize,
    screen_height: usize,
    border_thickness: u32,
    border_gap: u32,
) {
    let border = border_thickness as usize;
    let border_double = border * 2;
    let gap = border_gap as usize;
    let gap_double = gap * 2;
    let clients_length = clients.iter().filter(|&c| c.visible).count();

    for (i, client) in clients.iter().enumerate() {
        if !client.visible {
            continue;
        }

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
