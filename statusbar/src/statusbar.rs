use crate::config::get_config;
use crate::draw::Draw;
use crate::screen::get_screen;
use std::sync::Arc;
use actix::{Actor, AsyncContext, StreamHandler, Supervised, SystemService};

pub struct StatusBar {
    conn: Arc<xcb_util::ewmh::Connection>,
    draw: Draw,
}

impl Default for StatusBar {
    fn default() -> Self {
        let (conn, _) = xcb::Connection::connect(None)
            .expect("Unable to access your display. Check your DISPLAY environment variable.");

        let conn = xcb_util::ewmh::Connection::connect(conn)
            .map_err(|(e, _)| e)
            .expect("Unable to create EWMH connection.");

        let config = Arc::new(get_config());

        let screen = get_screen(&conn);

        xcb::change_window_attributes(&conn, screen.root(), &[
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_PROPERTY_CHANGE)
        ]);

        let window = conn.generate_id();

        xcb::create_window(
            &conn,
            xcb::WINDOW_CLASS_COPY_FROM_PARENT as u8,
            window,
            screen.root(),
            config.margin as i16, config.margin as i16,
            screen.width_in_pixels() - config.margin * 2, config.height,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[
                (xcb::CW_BACK_PIXEL, config.background_color),
                (xcb::CW_OVERRIDE_REDIRECT, 0),
            ],
        );

        xcb_util::ewmh::set_wm_window_type(&conn, window, &[conn.WM_WINDOW_TYPE_DOCK()]);

        xcb_util::ewmh::set_wm_strut_partial(&conn, window, xcb_util::ewmh::StrutPartial {
            left: 0,
            right: 0,
            top: (config.height + config.margin) as u32,
            bottom: 0,
            left_start_y: 0,
            left_end_y: 0,
            right_start_y: 0,
            right_end_y: 0,
            top_start_x: 0,
            top_end_x: 0,
            bottom_start_x: 0,
            bottom_end_x: 0,
        });

        xcb::map_window(&conn, window);

        conn.flush();

        let conn = Arc::new(conn);

        let draw = Draw::new(conn.clone(), config.clone(), window);

        Self {
            conn,
            draw,
        }
    }
}

impl Actor for StatusBar {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut actix::Context<Self>) {
        self.draw.workspaces();

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

impl Supervised for StatusBar {}
impl SystemService for StatusBar {}

impl StreamHandler<Option<xcb::GenericEvent>> for StatusBar {
    fn handle(&mut self, event: Option<xcb::GenericEvent>, _ctx: &mut actix::Context<Self>) {
        if let Some(e) = event {
            let conn = self.conn.clone();

            match e.response_type() {
                xcb::PROPERTY_NOTIFY => self.draw.workspaces(),
                // Events we do not care about
                _ => (),
            };

            conn.flush();
        }
    }
}
