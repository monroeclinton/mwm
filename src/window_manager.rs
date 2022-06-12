use crate::config::{Config, get_config};
use crate::event::EventContext;
use crate::key::grab_key;
use crate::listeners;
use crate::screen::get_screen;
use std::sync::Arc;
use actix::{Actor, AsyncContext, StreamHandler, Supervised, SystemService};

pub struct WindowManager {
    config: Arc<Config>,
    conn: Arc<xcb_util::ewmh::Connection>,
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
            conn.CURRENT_DESKTOP(),
            conn.ACTIVE_WINDOW(),
        ]);

        Self {
            config: Arc::new(get_config()),
            conn: Arc::new(conn),
        }
    }
}

impl Actor for WindowManager {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut actix::Context<Self>) {
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
    fn handle(&mut self, event: Option<xcb::GenericEvent>, _ctx: &mut actix::Context<Self>) {
        if let Some(e) = event {
            let config = self.config.clone();
            let conn = self.conn.clone();

            actix::spawn(async move {
                match e.response_type() {
                    xcb::CLIENT_MESSAGE => listeners::on_client_message(EventContext {
                        config,
                        conn: conn.clone(),
                        event: unsafe {
                            std::mem::transmute::<xcb::GenericEvent, xcb::ClientMessageEvent>(e)
                        },
                    }).await,
                    xcb::KEY_PRESS => listeners::on_key_press(EventContext {
                        config,
                        conn: conn.clone(),
                        event: unsafe {
                            std::mem::transmute::<xcb::GenericEvent, xcb::KeyPressEvent>(e)
                        },
                    }).await,
                    xcb::CONFIGURE_REQUEST => listeners::on_configure_request(EventContext {
                        config,
                        conn: conn.clone(),
                        event: unsafe {
                            std::mem::transmute::<xcb::GenericEvent, xcb::ConfigureRequestEvent>(e)
                        },
                    }).await,
                    xcb::MAP_REQUEST => listeners::on_map_request(EventContext {
                        config,
                        conn: conn.clone(),
                        event: unsafe {
                            std::mem::transmute::<xcb::GenericEvent, xcb::MapRequestEvent>(e)
                        },
                    }).await,
                    xcb::PROPERTY_NOTIFY => listeners::on_property_notify(EventContext {
                        config,
                        conn: conn.clone(),
                        event: unsafe {
                            std::mem::transmute::<xcb::GenericEvent, xcb::PropertyNotifyEvent>(e)
                        },
                    }).await,
                    xcb::ENTER_NOTIFY => listeners::on_enter_notify(EventContext {
                        config,
                        conn: conn.clone(),
                        event: unsafe {
                            std::mem::transmute::<xcb::GenericEvent, xcb::EnterNotifyEvent>(e)
                        },
                    }).await,
                    xcb::UNMAP_NOTIFY => listeners::on_unmap_notify(EventContext {
                        config,
                        conn: conn.clone(),
                        event: unsafe {
                            std::mem::transmute::<xcb::GenericEvent, xcb::UnmapNotifyEvent>(e)
                        },
                    }).await,
                    xcb::DESTROY_NOTIFY => listeners::on_destroy_notify(EventContext {
                        config,
                        conn: conn.clone(),
                        event: unsafe {
                            std::mem::transmute::<xcb::GenericEvent, xcb::DestroyNotifyEvent>(e)
                        },
                    }).await,
                    // Events we do not care about
                    _ => (),
                };

                conn.flush();
            });
        }
    }
}
