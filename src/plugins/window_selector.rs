use crate::config::Config;
use crate::client::{Client, Clients, GetClients};
use crate::event::{EventContext, EnterNotifyEvent};
use actix::{Actor, ActorFutureExt, Handler, ResponseActFuture, Supervised, SystemService, WrapFuture};
use anyhow::Result;

pub struct WindowSelector {
    active_window: xcb::Window,
}

impl Default for WindowSelector {
    fn default() -> Self {
        Self {
            active_window: 0,
        }
    }
}

impl Actor for WindowSelector {
    type Context = actix::Context<Self>;
}

impl Supervised for WindowSelector {}
impl SystemService for WindowSelector {}

impl Handler<EventContext<EnterNotifyEvent>> for WindowSelector {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<EnterNotifyEvent>, _ctx: &mut Self::Context) -> Self::Result {
        self.active_window = ectx.event.window;

        Clients::from_registry()
            .send(GetClients)
            .into_actor(self)
            .map(move |result, _actor, _ctx| {
                let clients = result?;

                set_active_window(
                    &ectx.conn,
                    &ectx.config,
                    &clients,
                    ectx.event.window
                );

                Ok(())
            })
            .boxed_local()
    }
}

fn set_active_window(
    conn: &xcb::Connection,
    config: &Config,
    clients: &Vec<Client>,
    window: xcb::Window,
) {
    let active_border = config.active_border;
    let inactive_border = config.inactive_border;

    xcb::set_input_focus(
        conn,
        xcb::INPUT_FOCUS_PARENT as u8,
        window,
        xcb::CURRENT_TIME,
    );

    xcb::change_window_attributes(conn, window, &[(xcb::CW_BORDER_PIXEL, active_border)]);

    for client in clients.iter() {
        if client.window != window {
            xcb::change_window_attributes(
                conn,
                client.window,
                &[(xcb::CW_BORDER_PIXEL, inactive_border)],
            );
        }
    }
}
