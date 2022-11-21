use crate::config::{Config, get_config};
use crate::surface::Surface;
use crate::screen::get_screen;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio::time::{Duration, interval};

pub struct StatusBar {
    conn: Arc<xcb_util::ewmh::Connection>,
    config: Config,
    surface: Surface,
}

impl StatusBar {
    pub fn new() -> Self {
        let (conn, _) = xcb::Connection::connect(None)
            .expect("Unable to access your display. Check your DISPLAY environment variable.");

        let conn = xcb_util::ewmh::Connection::connect(conn)
            .map_err(|(e, _)| e)
            .expect("Unable to create EWMH connection.");

        let config = get_config();

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

        // Uses xcb connection which will live length of program.
        let cairo_conn = unsafe {
            cairo::XCBConnection::from_raw_none((*conn.get_raw_conn()).connection as _)
        };

        // I wish there was a better way to do this
        // https://xcb.freedesktop.org/xlibtoxcbtranslationguide/
        // https://tronche.com/gui/x/xlib/window/visual-types.html
        let mut visual_type = screen.allowed_depths()
            .find_map(|depth| {
                depth.visuals().find(|visual| screen.root_visual() == visual.visual_id())
            })
            .expect("Unable to find visual type of screen.");

        let visual = unsafe {
            cairo::XCBVisualType::from_raw_none(&mut visual_type.base as *mut _ as _)
        };

        let drawable = cairo::XCBDrawable(window);
        let surface = cairo::XCBSurface::create(
            &cairo_conn,
            &drawable,
            &visual,
            screen.width_in_pixels() as i32,
            40
        ).expect("Unable to create Cairo surface.");

        let bar_height = config.height as f64;
        let bar_width = screen.width_in_pixels() as f64 - (config.margin * 2) as f64;
        let surface = Surface::new(surface, bar_width, bar_height);

        Self {
            conn: Arc::new(conn),
            config,
            surface,
        }
    }

    pub async fn run(mut self) {
        let (tx, mut rx) = channel(100);

        let event_tx = tx.clone();
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            loop {
                conn.wait_for_event();
                event_tx.try_send(()).unwrap();
            }
       });

        let interval_tx = tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));

            loop {
                interval.tick().await;
                let _ = interval_tx.send(()).await;
            }
        });

        loop {
            self.draw();

            rx.recv().await;
        }
    }

    pub fn draw(&mut self) {
        // Draw title
        let reply = xcb_util::ewmh::get_active_window(
            &self.conn,
            0
        ).get_reply();

        let window_name = if let Ok(active_window) = reply {
            let reply = xcb_util::ewmh::get_wm_name(
                &self.conn,
                active_window
            ).get_reply();

            match reply {
                Ok(name) => Some(name.string().to_string()),
                _ => None,
            }
        } else {
            None
        };

        self.surface.bar_title(&self.config, window_name);

        // Draw workspaces
        let reply = xcb_util::ewmh::get_desktop_names(
            &self.conn,
            0,
        ).get_reply().expect("Unable to get desktop names.");

        let workspaces = reply.strings();

        let active_workspace = xcb_util::ewmh::get_current_desktop(
            &self.conn,
            0
        ).get_reply().unwrap_or(0) as usize;

        self.surface.workspaces(&self.config, workspaces, active_workspace);

        // Draw info blocks
        self.surface.draw_info(&self.config);

        self.surface.flush();
        self.conn.flush();
    }
}
