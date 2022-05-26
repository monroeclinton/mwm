use crate::config::{Config, get_config};
use crate::client::Client;
use crate::event::{EventContext, EnterNotifyEvent};
use crate::window_manager::{GetClients, WindowManager};
use std::collections::VecDeque;
use actix::{Actor, ActorFutureExt, Handler, ResponseActFuture, Supervised, SystemService};
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

        let config = get_config();
        let clients = actix::fut::wrap_future::<_, Self>(get_clients());

        let handle_enter = clients.map(move |result, _actor, _ctx| {
            let clients = result?;

            set_active_window(
                &ectx.conn,
                &config,
                &clients,
                ectx.event.window
            );

            Ok(())
        });

        Box::pin(handle_enter)
    }
}

async fn get_clients() -> Result<VecDeque<Client>> {
    WindowManager::from_registry()
        .send(GetClients)
        .await?
}

fn set_active_window(
    conn: &xcb::Connection,
    config: &Config,
    clients: &VecDeque<Client>,
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
