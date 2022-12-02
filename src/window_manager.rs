use crate::client::Clients;
use crate::config::{get_config, Config};
use crate::event::EventContext;
use crate::handler::Handler;
use crate::key::grab_key;
use crate::screen::get_screen;
use std::sync::{Arc, Mutex};

pub struct WindowManager {
    clients: Arc<Mutex<Clients>>,
    config: Arc<Config>,
    conn: Arc<xcb_util::ewmh::Connection>,
    cursor: xcb::Cursor,
}

impl WindowManager {
    pub fn new() -> Self {
        let (conn, screen) = xcb::Connection::connect(None)
            .expect("Unable to access your display. Check your DISPLAY environment variable.");

        let conn = xcb_util::ewmh::Connection::connect(conn)
            .map_err(|(e, _)| e)
            .expect("Unable to create EWMH connection.");

        xcb_util::ewmh::set_supported(
            &conn,
            screen,
            &[
                conn.SUPPORTED(),
                conn.CLIENT_LIST(),
                conn.NUMBER_OF_DESKTOPS(),
                conn.DESKTOP_NAMES(),
                conn.CURRENT_DESKTOP(),
                conn.ACTIVE_WINDOW(),
            ],
        );

        let conn = Arc::new(conn);
        let config = Arc::new(get_config());

        let clients = Arc::new(Mutex::new(Clients::new(conn.clone(), config.clone())));

        let cursor = xcb_util::cursor::create_font_cursor(&conn, xcb_util::cursor::LEFT_PTR);

        Self {
            clients,
            config,
            conn,
            cursor,
        }
    }

    pub fn run(self) {
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
                screen.root(),
            );

            grab_key(
                &self.conn,
                self.config.workspace_move_window_modifier,
                x11::keysym::XK_0 + workspace as u32,
                screen.root(),
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
            std::process::Command::new(program).spawn().unwrap();
        }

        let values = [(xcb::CW_CURSOR, self.cursor)];

        let cookie = xcb::change_window_attributes_checked(&self.conn, screen.root(), &values);

        if cookie.request_check().is_err() {
            panic!("Unable to set cursor icon.")
        }

        tracing::info!("Started window manager.");

        loop {
            if let Some(event) = self.conn.wait_for_event() {
                let clients = self.clients.clone();
                let config = self.config.clone();
                let conn = self.conn.clone();

                tokio::spawn(Self::handle(clients, config, conn, event));
            }
        }
    }

    #[tracing::instrument(skip_all, name = "event_handle")]
    async fn handle(
        clients: Arc<Mutex<Clients>>,
        config: Arc<Config>,
        conn: Arc<xcb_util::ewmh::Connection>,
        event: xcb::GenericEvent,
    ) {
        let mut handler = Handler::default();

        let response_type = event.response_type() & !0x80;

        tracing::debug!("response_type={}", response_type);

        match response_type {
            xcb::CLIENT_MESSAGE => handler.on_client_message(EventContext {
                clients,
                config,
                conn: conn.clone(),
                event: unsafe {
                    std::mem::transmute::<xcb::GenericEvent, xcb::ClientMessageEvent>(event)
                },
            }),
            xcb::KEY_PRESS => handler.on_key_press(EventContext {
                clients,
                config,
                conn: conn.clone(),
                event: unsafe {
                    std::mem::transmute::<xcb::GenericEvent, xcb::KeyPressEvent>(event)
                },
            }),
            xcb::CONFIGURE_REQUEST => handler.on_configure_request(EventContext {
                clients,
                config,
                conn: conn.clone(),
                event: unsafe {
                    std::mem::transmute::<xcb::GenericEvent, xcb::ConfigureRequestEvent>(event)
                },
            }),
            xcb::MAP_REQUEST => handler.on_map_request(EventContext {
                clients,
                config,
                conn: conn.clone(),
                event: unsafe {
                    std::mem::transmute::<xcb::GenericEvent, xcb::MapRequestEvent>(event)
                },
            }),
            xcb::PROPERTY_NOTIFY => handler.on_property_notify(EventContext {
                clients,
                config,
                conn: conn.clone(),
                event: unsafe {
                    std::mem::transmute::<xcb::GenericEvent, xcb::PropertyNotifyEvent>(event)
                },
            }),
            xcb::ENTER_NOTIFY => handler.on_enter_notify(EventContext {
                clients,
                config,
                conn: conn.clone(),
                event: unsafe {
                    std::mem::transmute::<xcb::GenericEvent, xcb::EnterNotifyEvent>(event)
                },
            }),
            xcb::UNMAP_NOTIFY => handler.on_unmap_notify(EventContext {
                clients,
                config,
                conn: conn.clone(),
                event: unsafe {
                    std::mem::transmute::<xcb::GenericEvent, xcb::UnmapNotifyEvent>(event)
                },
            }),
            xcb::DESTROY_NOTIFY => handler.on_destroy_notify(EventContext {
                clients,
                config,
                conn: conn.clone(),
                event: unsafe {
                    std::mem::transmute::<xcb::GenericEvent, xcb::DestroyNotifyEvent>(event)
                },
            }),
            // Events we do not care about
            _ => (),
        };

        conn.flush();
    }
}
