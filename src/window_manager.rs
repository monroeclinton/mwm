use crate::client::Client;
use crate::config::get_config;
use crate::event;
use crate::plugin::EventContext;
use crate::plugins;
use std::collections::VecDeque;
use std::sync::Arc;
use actix::{Actor, AsyncContext, Handler, Message, StreamHandler, Supervised, SystemService};
use anyhow::Result;

pub struct WindowManager {
    conn: Arc<xcb::Connection>,
    clients: VecDeque<Client>,
}

impl WindowManager {
    fn on_key_press(&self, context: EventContext<event::KeyPressEvent>) {
        plugins::Commands::from_registry().do_send(context.clone());
    }

    fn on_configure_request(&self, context: EventContext<event::ConfigureRequestEvent>) {
        plugins::ConfigureWindow::from_registry().do_send(context);
    }

    fn on_map_request(&self, context: EventContext<event::MapRequestEvent>) {
        plugins::MapWindow::from_registry().do_send(context.clone());
        plugins::WindowSizer::from_registry().do_send(context.clone());
    }

    fn on_enter_notify(&self, _context: EventContext<event::EnterNotifyEvent>) {
        plugins::WindowSelector::from_registry().do_send(_context.clone());
    }

    fn on_unmap_notify(&self, _context: EventContext<event::UnmapNotifyEvent>) {
    }
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
            match e.response_type() {
                xcb::KEY_PRESS => self.on_key_press(EventContext {
                    conn: Arc::clone(&self.conn),
                    event: event::KeyPressEvent::from(unsafe { xcb::cast_event(&e) }),
                }),
                xcb::CONFIGURE_REQUEST => self.on_configure_request(EventContext {
                    conn: Arc::clone(&self.conn),
                    event: event::ConfigureRequestEvent::from(unsafe { xcb::cast_event(&e) }),
                }),
                xcb::MAP_REQUEST => self.on_map_request(EventContext {
                    conn: Arc::clone(&self.conn),
                    event: event::MapRequestEvent::from(unsafe { xcb::cast_event(&e) }),
                }),
                xcb::ENTER_NOTIFY => self.on_enter_notify(EventContext {
                    conn: Arc::clone(&self.conn),
                    event: event::EnterNotifyEvent::from(unsafe { xcb::cast_event(&e) }),
                }),
                xcb::UNMAP_NOTIFY => self.on_unmap_notify(EventContext {
                    conn: Arc::clone(&self.conn),
                    event: event::UnmapNotifyEvent::from(unsafe { xcb::cast_event(&e) }),
                }),
                // Events we do not care about
                _ => (),
            };
        }

        self.conn.flush();
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
