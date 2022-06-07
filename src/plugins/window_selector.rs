use crate::config::{Actions, Config};
use crate::client::{Client, Clients, GetClients};
use crate::event::EventContext;
use std::sync::Arc;
use actix::{Actor, ActorFutureExt, AsyncContext, Handler, Message, ResponseActFuture, Supervised, SystemService, WrapFuture};
use anyhow::Result;

pub struct WindowSelector {
    active_window: xcb::Window,
}

impl WindowSelector {
    fn set_active_window(
        &mut self,
        conn: Arc<xcb_util::ewmh::Connection>,
        config: Arc<Config>,
        window: xcb::Window
    ) -> ResponseActFuture<Self, Result<()>> {
        self.active_window = window;

        Clients::from_registry()
            .send(GetClients)
            .into_actor(self)
            .map(move |result, _actor, ctx| {
                let clients = result?;

                ctx.notify(SetActiveWindow {
                    conn,
                    config,
                    clients,
                    window,
                });

                Ok(())
            })
            .boxed_local()
    }
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

impl Handler<EventContext<xcb::ClientMessageEvent>> for WindowSelector {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::ClientMessageEvent>, _ctx: &mut Self::Context) -> Self::Result {
        if ectx.event.type_() == ectx.conn.ACTIVE_WINDOW() {
            self.set_active_window(ectx.conn, ectx.config, ectx.event.window())
        } else {
            Box::pin(actix::fut::wrap_future::<_, Self>(async {
                Ok(())
            }))
        }
    }
}

impl Handler<EventContext<xcb::KeyPressEvent>> for WindowSelector {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::KeyPressEvent>, _ctx: &mut Self::Context) -> Self::Result {
        let active_window = self.active_window;

        Clients::from_registry()
            .send(GetClients)
            .into_actor(self)
            .map(move |result, _actor, ctx| {
                let clients = result?;
                let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);

                for action in ectx.config.actions.iter() {
                    let keycode = key_symbols
                        .get_keycode(action.keysym)
                        .next()
                        .expect("Unknown keycode found in window_selector plugin.");

                    if keycode == ectx.event.detail() && action.modifier == ectx.event.state() {
                        if let Some(window) = move_window(&active_window, &action.action, &clients) {
                            ctx.notify(SetActiveWindow {
                                conn: ectx.conn.clone(),
                                config: ectx.config.clone(),
                                clients: clients.clone(),
                                window,
                            });
                        }
                    }
                }

                Ok(())
            })
            .boxed_local()
    }
}

impl Handler<EventContext<xcb::EnterNotifyEvent>> for WindowSelector {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::EnterNotifyEvent>, _ctx: &mut Self::Context) -> Self::Result {
        self.set_active_window(ectx.conn, ectx.config, ectx.event.event())
    }
}

struct SetActiveWindow {
    conn: Arc<xcb_util::ewmh::Connection>,
    config: Arc<Config>,
    clients: Vec<Client>,
    window: xcb::Window,
}

impl Message for SetActiveWindow {
    type Result = ();
}

impl Handler<SetActiveWindow> for WindowSelector {
    type Result = ();

    fn handle(&mut self, msg: SetActiveWindow, _ctx: &mut Self::Context) -> Self::Result {
        self.active_window = msg.window;

        set_active_window(
            &msg.conn,
            &msg.config,
            &msg.clients,
            msg.window
        );
    }
}

fn move_window(
    active_window: &xcb::Window,
    action: &Actions,
    clients: &[Client]
) -> Option<xcb::Window> {
    let pos = clients
        .iter()
        .position(|c| &c.window == active_window)
        .unwrap_or(0);

    let new_window_pos = match action {
        Actions::SelectLeftWindow => {
            if pos >= clients.len() - 1 {
                0
            } else {
                pos + 1
            }
        },
        Actions::SelectRightWindow => {
            if pos == 0 && clients.len() == 0 {
                0
            } else if pos == 0 && clients.len() > 0 {
                clients.len() - 1
            } else {
                pos - 1
            }
        },
    };

    if let Some(client) = clients.get(new_window_pos) {
        Some(client.window)
    } else {
        None
    }
}

fn set_active_window(
    conn: &xcb_util::ewmh::Connection,
    config: &Config,
    clients: &[Client],
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

    xcb_util::ewmh::set_active_window(&conn, 0, window);
}
