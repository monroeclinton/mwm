use crate::client::Clients;
use crate::config::{Config, get_config};
use crate::event::EventContext;
use crate::key::grab_key;
use crate::listener::Listener;
use crate::screen::get_screen;
use std::sync::Arc;
use actix::{Actor, Addr, AsyncContext, Context, StreamHandler, Supervised, SystemService};

pub struct WindowManager {
    clients: Addr<Clients>,
    config: Arc<Config>,
    conn: Arc<xcb_util::ewmh::Connection>,
    listener: Listener,
}

impl Default for WindowManager {
    fn default() -> Self {
        let (conn, screen) = xcb::Connection::connect(None)
            .expect("Unable to access your display. Check your DISPLAY environment variable.");

        let conn = xcb_util::ewmh::Connection::connect(conn)
            .map_err(|(e, _)| e)
            .expect("Unable to create EWMH connection.");

        xcb_util::ewmh::set_supported(&conn, screen, &[
            conn.SUPPORTED(),
            conn.CLIENT_LIST(),
            conn.NUMBER_OF_DESKTOPS(),
            conn.DESKTOP_NAMES(),
            conn.CURRENT_DESKTOP(),
            conn.ACTIVE_WINDOW(),
        ]);

        let conn = Arc::new(conn);
        let config = Arc::new(get_config());

        let clients = Clients::new(conn.clone(), config.clone())
            .start();

        Self {
            clients,
            config,
            conn,
            listener: Listener::default(),
        }
    }
}

impl Actor for WindowManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let screen = get_screen(&self.conn);

        for command in &self.config.commands {
            grab_key(&self.conn, command.modifier, command.keysym, screen.root());
        }

        for action in &self.config.actions {
            grab_key(&self.conn, action.modifier, action.keysym, screen.root());
        }

        for workspace in 1..=9 {
            grab_key(
                &self.conn,
                self.config.workspace_modifier,
                x11::keysym::XK_0 + workspace as u32,
                screen.root()
            );

            grab_key(
                &self.conn,
                self.config.workspace_move_window_modifier,
                x11::keysym::XK_0 + workspace as u32,
                screen.root()
            );
        }

        xcb_util::ewmh::set_number_of_desktops(&self.conn, 0, 9);
        xcb_util::ewmh::set_current_desktop(&self.conn, 0, 1);

        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY,
        )];

        let cookie = xcb::change_window_attributes_checked(&self.conn, screen.root(), &values);

        if cookie.request_check().is_err() {
            panic!("Unable to change window attributes. Is another window manager running?")
        }

        for program in &self.config.autostart {
            std::process::Command::new(program)
                .spawn()
                .unwrap();
        }

        let events = futures::stream::unfold(self.conn.clone(), |c| async move {
            let conn = c.clone();
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
    fn handle(&mut self, event: Option<xcb::GenericEvent>, _ctx: &mut Context<Self>) {
        if let Some(e) = event {
            let clients = self.clients.clone();
            let config = self.config.clone();
            let conn = self.conn.clone();

            match e.response_type() {
                xcb::CLIENT_MESSAGE => self.listener.on_client_message(EventContext {
                    clients,
                    config,
                    conn: conn.clone(),
                    event: unsafe {
                        std::mem::transmute::<xcb::GenericEvent, xcb::ClientMessageEvent>(e)
                    },
                }),
                xcb::KEY_PRESS => self.listener.on_key_press(EventContext {
                    clients,
                    config,
                    conn: conn.clone(),
                    event: unsafe {
                        std::mem::transmute::<xcb::GenericEvent, xcb::KeyPressEvent>(e)
                    },
                }),
                xcb::CONFIGURE_REQUEST => self.listener.on_configure_request(EventContext {
                    clients,
                    config,
                    conn: conn.clone(),
                    event: unsafe {
                        std::mem::transmute::<xcb::GenericEvent, xcb::ConfigureRequestEvent>(e)
                    },
                }),
                xcb::MAP_REQUEST => self.listener.on_map_request(EventContext {
                    clients,
                    config,
                    conn: conn.clone(),
                    event: unsafe {
                        std::mem::transmute::<xcb::GenericEvent, xcb::MapRequestEvent>(e)
                    },
                }),
                xcb::PROPERTY_NOTIFY => self.listener.on_property_notify(EventContext {
                    clients,
                    config,
                    conn: conn.clone(),
                    event: unsafe {
                        std::mem::transmute::<xcb::GenericEvent, xcb::PropertyNotifyEvent>(e)
                    },
                }),
                xcb::ENTER_NOTIFY => self.listener.on_enter_notify(EventContext {
                    clients,
                    config,
                    conn: conn.clone(),
                    event: unsafe {
                        std::mem::transmute::<xcb::GenericEvent, xcb::EnterNotifyEvent>(e)
                    },
                }),
                xcb::UNMAP_NOTIFY => self.listener.on_unmap_notify(EventContext {
                    clients,
                    config,
                    conn: conn.clone(),
                    event: unsafe {
                        std::mem::transmute::<xcb::GenericEvent, xcb::UnmapNotifyEvent>(e)
                    },
                }),
                xcb::DESTROY_NOTIFY => self.listener.on_destroy_notify(EventContext {
                    clients,
                    config,
                    conn: conn.clone(),
                    event: unsafe {
                        std::mem::transmute::<xcb::GenericEvent, xcb::DestroyNotifyEvent>(e)
                    },
                }),
                // Events we do not care about
                _ => (),
            };

            conn.flush();
        }
    }
}
