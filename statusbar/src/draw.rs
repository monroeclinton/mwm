use crate::config::Config;
use std::sync::Arc;

pub struct Draw {
    xcb_conn: Arc<xcb_util::ewmh::Connection>,
    config: Arc<Config>,
    surface: cairo::XCBSurface,
}

impl Draw {
    pub fn new(
        xcb_conn: Arc<xcb_util::ewmh::Connection>,
        config: Arc<Config>,
        window: xcb::Window
    ) -> Self {
        // Uses statusbar xcb connection which will live length of program.
        let cairo_conn = unsafe {
            cairo::XCBConnection::from_raw_none((*xcb_conn.get_raw_conn()).connection as _)
        };

        let screen = xcb_conn.get_setup().roots().next()
            .expect("Unable to find a screen.");

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

        Self {
            xcb_conn,
            config,
            surface
        }
    }

    pub fn workspaces(&self) {
        let workspaces = xcb_util::ewmh::get_number_of_desktops(
            &self.xcb_conn,
            0,
        ).get_reply().unwrap_or(0);

        let active_workspace = xcb_util::ewmh::get_current_desktop(
            &self.xcb_conn,
            0
        ).get_reply().unwrap_or(0);

        let context = cairo::Context::new(&self.surface)
            .expect("Unable to find context of statusbar surface.");

        let font_face = cairo::FontFace::toy_create(
            self.config.font_family.as_str(),
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal
        ).expect("Unable to create font face in statusbar.");

        let mut offset = 10.0;

        for i in 1..=workspaces {
            let workspace = i.to_string();
            let extents = context.text_extents(workspace.as_str())
                .expect("Unable to find text text extents of statusbar workspace.");

            context.set_font_face(&font_face);
            context.set_font_size(self.config.font_size as f64);

            if i == active_workspace {
                context.set_source_rgb(255.0, 255.0, 255.0);
            } else {
                context.set_source_rgb(0.0, 0.0, 0.0);
            }

            context.move_to(offset, (self.config.height as f64 / 2.0) + (extents.height / 2.0));
            context.show_text(workspace.as_str())
                .expect("Cannot position text on surface in statusbar.");

            offset += extents.width + 10.0;
        }

        self.surface.flush();
        self.xcb_conn.flush();
    }
}
