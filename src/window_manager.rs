use crate::config::{Config, get_config};
use crate::event::{
    EventContext, KeyPressEvent, ConfigureRequestEvent, MapRequestEvent,
    EnterNotifyEvent, UnmapNotifyEvent
};
use crate::key::grab_key;
use crate::listeners;
use std::sync::Arc;
use actix::{Actor, AsyncContext, StreamHandler, Supervised, SystemService};

pub struct WindowManager {
    config: Config,
    conn: Arc<xcb::Connection>,
}

impl Default for WindowManager {
    fn default() -> Self {
        let (conn, _) = xcb::Connection::connect(None)
            .expect("Unable to access your display. Check your DISPLAY environment variable.");

        Self {
            config: get_config(),
            conn: Arc::new(conn),
        }
    }
}

impl Actor for WindowManager {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut actix::Context<Self>) {
        let screen = self.conn.get_setup().roots().next()
            .expect("Unable to find a screen.");

        for command in &self.config.commands {
            grab_key(&self.conn, command.modifier, command.keysym, screen.root());
        }

        for action in &self.config.actions {
            grab_key(&self.conn, action.modifier, action.keysym, screen.root());
        }

        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY,
        )];

        let cookie = xcb::change_window_attributes_checked(&self.conn, screen.root(), &values);

        if cookie.request_check().is_err() {
            panic!("Unable to change window attributes. Is another window manager running?")
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
                    xcb::KEY_PRESS => listeners::on_key_press(EventContext {
                        config,
                        conn: conn.clone(),
                        event: KeyPressEvent::from(unsafe { xcb::cast_event(&e) }),
                    }).await,
                    xcb::CONFIGURE_REQUEST => listeners::on_configure_request(EventContext {
                        config,
                        conn: conn.clone(),
                        event: ConfigureRequestEvent::from(unsafe { xcb::cast_event(&e) }),
                    }).await,
                    xcb::MAP_REQUEST => listeners::on_map_request(EventContext {
                        config,
                        conn: conn.clone(),
                        event: MapRequestEvent::from(unsafe { xcb::cast_event(&e) }),
                    }).await,
                    xcb::ENTER_NOTIFY => listeners::on_enter_notify(EventContext {
                        config,
                        conn: conn.clone(),
                        event: EnterNotifyEvent::from(unsafe { xcb::cast_event(&e) }),
                    }).await,
                    xcb::UNMAP_NOTIFY => listeners::on_unmap_notify(EventContext {
                        config,
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
