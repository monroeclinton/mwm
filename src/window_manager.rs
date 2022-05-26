use crate::client::Client;
use crate::config::get_config;
use crate::event::{
    EventContext, KeyPressEvent, ConfigureRequestEvent, MapRequestEvent,
    EnterNotifyEvent, UnmapNotifyEvent
};
use crate::listeners;
use std::collections::VecDeque;
use std::sync::Arc;
use actix::{Actor, AsyncContext, Handler, Message, StreamHandler, Supervised, SystemService};
use anyhow::Result;

pub struct WindowManager {
    conn: Arc<xcb::Connection>,
    clients: VecDeque<Client>,
}

impl Default for WindowManager {
    fn default() -> Self {
        let (conn, _) = xcb::Connection::connect(None)
            .expect("Unable to access your display. Check your DISPLAY environment variable.");

        let screen = match conn.get_setup().roots().next() {
            Some(s) => s,
            None => panic!("Unable to find a screen."),
        };

        let config = get_config();

        let key_symbols = xcb_util::keysyms::KeySymbols::new(&conn);
        for command in config.commands {
            match key_symbols.get_keycode(command.keysym).next() {
                Some(keycode) => {
                    xcb::grab_key(
                        &conn,
                        false,
                        screen.root(),
                        command.modifier,
                        keycode,
                        xcb::GRAB_MODE_ASYNC as u8,
                        xcb::GRAB_MODE_ASYNC as u8,
                    );
                }
                _ => {
                    dbg!("Failed to find keycode for keysym: {}", command.keysym);
                }
            }
        }

        drop(key_symbols);

        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY,
        )];

        let cookie = xcb::change_window_attributes_checked(&conn, screen.root(), &values);

        match cookie.request_check() {
            Ok(_) => (),
            Err(_) => {
                panic!("Unable to change window attributes. Is another window manager running?")
            }
        }

        Self {
            conn: Arc::new(conn),
            clients: VecDeque::new(),
        }
    }
}

impl Actor for WindowManager {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut actix::Context<Self>) {
        let events = futures::stream::unfold(Arc::clone(&self.conn), |c| async move {
            let conn = Arc::clone(&c);
            let event = tokio::task::spawn_blocking(move || {
                conn.wait_for_event()
            }).await.unwrap();

            Some((event, c))
        });

        ctx.add_stream(events);
    }
}

impl Supervised for WindowManager {}
impl SystemService for WindowManager {}

impl StreamHandler<Option<xcb::GenericEvent>> for WindowManager {
    fn handle(&mut self, event: Option<xcb::GenericEvent>, _ctx: &mut actix::Context<Self>) {
        if let Some(e) = event {
            let conn = self.conn.clone();

            actix::spawn(async move {
                match e.response_type() {
                    xcb::KEY_PRESS => listeners::on_key_press(EventContext {
                        conn: conn.clone(),
                        event: KeyPressEvent::from(unsafe { xcb::cast_event(&e) }),
                    }).await,
                    xcb::CONFIGURE_REQUEST => listeners::on_configure_request(EventContext {
                        conn: conn.clone(),
                        event: ConfigureRequestEvent::from(unsafe { xcb::cast_event(&e) }),
                    }).await,
                    xcb::MAP_REQUEST => listeners::on_map_request(EventContext {
                        conn: conn.clone(),
                        event: MapRequestEvent::from(unsafe { xcb::cast_event(&e) }),
                    }).await,
                    xcb::ENTER_NOTIFY => listeners::on_enter_notify(EventContext {
                        conn: conn.clone(),
                        event: EnterNotifyEvent::from(unsafe { xcb::cast_event(&e) }),
                    }).await,
                    xcb::UNMAP_NOTIFY => listeners::on_unmap_notify(EventContext {
                        conn: conn.clone(),
                        event: UnmapNotifyEvent::from(unsafe { xcb::cast_event(&e) }),
                    }).await,
                    // Events we do not care about
                    _ => (),
                };

                conn.flush();
            });
        }
    }
}

pub struct CreateClient {
    pub window: xcb::Window,
}

impl Message for CreateClient {
    type Result = Result<()>;
}

impl Handler<CreateClient> for WindowManager {
    type Result = Result<()>;

    fn handle(&mut self, msg: CreateClient, _ctx: &mut Self::Context) -> Self::Result {
        self.clients.push_front(Client {
            window: msg.window,
            visible: true,
        });

        Ok(())
    }
}

pub struct GetClients;

impl Message for GetClients {
    type Result = Result<VecDeque<Client>>;
}

impl Handler<GetClients> for WindowManager {
    type Result = Result<VecDeque<Client>>;

    fn handle(&mut self, _msg: GetClients, _ctx: &mut Self::Context) -> Self::Result {
        Ok(self.clients.clone())
    }
}
