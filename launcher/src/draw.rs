use crate::config::Config;
use std::rc::Rc;

pub struct Draw {
    xcb_conn: Rc<xcb::Connection>,
    config: Rc<Config>,
    surface: cairo::XCBSurface,
    window: xcb::Window,
}

impl Draw {
    pub fn new(
        xcb_conn: Rc<xcb::Connection>,
        config: Rc<Config>,
        window: xcb::Window
    ) -> Self {
        // Uses statusbar xcb connection which will live length of program.
        let cairo_conn = unsafe {
            cairo::XCBConnection::from_raw_none(xcb_conn.get_raw_conn() as _)
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
            config.width as i32,
            1
        ).expect("Unable to create Cairo surface.");

        Self {
            xcb_conn,
            config,
            surface,
            window,
        }
    }

    pub fn draw(&self, commands: &Vec<String>, selection_index: usize) {
        let title = "Command:";

        let item_height = (self.config.font_size + self.config.font_size / 2) as f64;
        let window_width = self.config.width as f64;
        let window_height = item_height * (commands.len() as f64 + 1.0);
        let font_size = self.config.font_size as f64;

        let screen = self.xcb_conn.get_setup().roots().next()
            .expect("Unable to find a screen.");

        let x = (screen.width_in_pixels() - self.config.width) / 2;
        let y = (screen.height_in_pixels() - window_height as u16) / 2;

        xcb::configure_window(
            &self.xcb_conn,
            self.window,
            &[
                (xcb::CONFIG_WINDOW_X as u16, x as u32),
                (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, window_height as u32),
            ],
        );

        self.xcb_conn.flush();

        self.surface.set_size(window_width as i32, window_height as i32)
            .expect("Unable to resize surface.");

        let context = cairo::Context::new(&self.surface)
            .expect("Unable to find context of launcher surface.");

        set_source_rgb(&context, self.config.background_color);
        context.paint().expect("Unable to clear surface.");

        let font_face = cairo::FontFace::toy_create(
            self.config.font_family.as_str(),
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal
        ).expect("Unable to create font face in statusbar.");

        let extents = context.text_extents(title)
            .expect("Unable to find text extents of title.");

        let title_width = extents.width * 2.0 + font_size;

        set_source_rgb(&context, self.config.background_active_color);

        context.rectangle(
            0.0,
            0.0,
            title_width,
            item_height,
        );

        context.fill()
            .expect("Unable to draw input box.");

        set_source_rgb(&context, self.config.font_active_color);

        context.set_font_face(&font_face);
        context.set_font_size(font_size);
        context.move_to(
            font_size,
            (item_height + extents.height) / 2.0
        );

        context.show_text(title)
            .expect("Cannot position title text.");

        for (i, command) in commands.iter().enumerate() {
            if i == selection_index {
                set_source_rgb(&context, self.config.background_active_color);

                context.rectangle(
                    0.0,
                    item_height * (i + 1) as f64,
                    window_width,
                    item_height,
                );

                context.fill()
                    .expect("Unable to draw selection box.");

                set_source_rgb(&context, self.config.font_active_color);
            } else {
                set_source_rgb(&context, self.config.font_color);
            }

            let extents = context.text_extents(command.as_str())
                .expect("Unable to find text extents of command.");

            context.move_to(
                font_size,
                (item_height * (i + 2) as f64) - extents.height / 2.0
            );

            context.show_text(command.as_str())
                .expect("Cannot position command text.");
        }

        self.surface.flush();
        self.xcb_conn.flush();
    }
}

fn set_source_rgb(context: &cairo::Context, color: u32) {
    context.set_source_rgb(
        (color >> 16 & 255) as f64 / 255.0,
        (color >> 8 & 255) as f64 / 255.0,
        (color & 255) as f64 / 255.0
    );
}
