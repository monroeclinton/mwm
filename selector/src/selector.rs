use crate::config::{Config, get_config};
use crate::keysym::Keysym;
use crate::surface::Surface;
use std::convert::TryFrom;
use anyhow::Result;

// Max items to show in list
const MAX_ITEMS: usize = 5;

pub struct Selector {
    conn: xcb::Connection,
    window: xcb::Window,
    config: Config,
    surface: Surface,
    commands: Vec<String>,
    search_input: String,
    selection_index: usize,
}

impl Selector {
    pub fn new(commands: Vec<String>) -> Self {
        let (conn, _) = xcb::Connection::connect(None)
            .expect("Unable to access your display. Check your DISPLAY enviroment variable.");

        let config = get_config();

        let screen = conn.get_setup().roots().next()
            .expect("Unable to find a screen.");

        let window = conn.generate_id();

        let screen_width = screen.width_in_pixels();
        let screen_height = screen.height_in_pixels();

        let x = (screen_width - config.width) / 2;
        let y = screen_height / 2;

        xcb::create_window(
            &conn,
            xcb::WINDOW_CLASS_COPY_FROM_PARENT as u8,
            window,
            screen.root(),
            x as i16, y as i16,
            config.width, 1,
            config.border_thickness,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[
                (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_KEY_PRESS),
                (xcb::CW_BACK_PIXEL, config.background_color),
                (xcb::CW_BORDER_PIXEL, config.border_color),
                (xcb::CW_OVERRIDE_REDIRECT, 1),
            ],
        );

        xcb::map_window(&conn, window);

        xcb::configure_window(&conn, window, &[
            (xcb::CW_EVENT_MASK as u16, xcb::EVENT_MASK_STRUCTURE_NOTIFY),
            (xcb::CONFIG_WINDOW_STACK_MODE as u16, xcb::STACK_MODE_ABOVE)
        ]);

        xcb::grab_keyboard(
            &conn,
            false,
            screen.root(),
            xcb::CURRENT_TIME,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
        );

        conn.flush();

        // Uses xcb connection which will live length of program.
        let cairo_conn = unsafe {
            cairo::XCBConnection::from_raw_none(conn.get_raw_conn() as _)
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
            config.width as i32,
            1
        ).expect("Unable to create Cairo surface.");

        let surface = Surface::new(surface);

        Self {
            conn,
            window,
            config,
            surface,
            commands,
            search_input: String::new(),
            selection_index: 0,
        }
    }

    pub fn run(mut self) {
        loop {
            self.draw();

            let event = match self.conn.wait_for_event() {
                Some(e) => e,
                _ => continue,
            };

            let status = match event.response_type() {
                xcb::KEY_PRESS => self.on_key_press(unsafe { xcb::cast_event(&event) }),
                xcb::MAP_REQUEST => Ok(()), // Requests should cause a redraw
                _ => continue,
            };

            if status.is_err() {
                println!(
                    "Error occurred processing event: {:?} - {:?}",
                    event.response_type(),
                    status
                );
            }

            self.conn.flush();
        }
    }

    fn on_key_press(&mut self, event: &xcb::KeyPressEvent) -> Result<()> {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&self.conn);

        if let Some(keycode) = key_symbols.get_keycode(self.config.close_keysym).next() {
            if event.detail() == keycode {
                std::process::exit(1);
            }
        }

        if let Some(keycode) = key_symbols.get_keycode(self.config.select_keysym).next() {
            if event.detail() == keycode {
                if let Some(command) = self.commands.get(self.selection_index) {
                    println!("{command}");
                    std::process::exit(1);
                }
            }
        }

        if let Some(keycode) = key_symbols.get_keycode(Keysym::XK_BackSpace as u32).next() {
            if event.detail() == keycode {
                // At least no memory allocation :*)
                // (I expect input to be not that long)
                self.search_input.pop();
                return Ok(());
            }
        }

        if let Some(keycode) = key_symbols.get_keycode(self.config.up_keysym).next() {
            if event.detail() == keycode && event.state() == self.config.modifier {
                if self.selection_index > 0 {
                    self.selection_index -= 1;
                } else {
                    self.selection_index = self.commands.len() - 1;
                }

                return Ok(());
            }
        }

        if let Some(keycode) = key_symbols.get_keycode(self.config.down_keysym).next() {
            if event.detail() == keycode && event.state() == self.config.modifier {
                if self.selection_index < self.commands.len() - 1 {
                    self.selection_index += 1;
                } else {
                    self.selection_index = 0;
                }

                return Ok(());
            }
        }

        let keysym = key_symbols.get_keysym(event.detail(), event.state() as i32);

        if let Ok(keysym) = Keysym::try_from(keysym) {
            self.search_input.push_str(keysym.to_string().as_str());
        }

        Ok(())
    }

    fn draw(&mut self) {
        let window_height = self.window_height() as f64;
        let item_height = self.item_height() as f64;

        self.configure_window();
        self.surface.clear_surface(&self.config);

        self.surface.draw_title(
            &self.config,
            &self.search_input,
            window_height,
            item_height
        );

        let commands = self.filtered_commands();

        let mut selection_index = 0;
        if self.selection_index > self.commands.len() - MAX_ITEMS {
            selection_index = self.selection_index - (self.commands.len() - MAX_ITEMS);
        }

        self.surface.draw_items(
            &commands,
            &self.config,
            item_height,
            selection_index
        );

        self.surface.flush();
        self.conn.flush();
    }

    fn configure_window(&self) {
        let window_height = self.window_height();

        xcb::configure_window(
            &self.conn,
            self.window,
            &[
                (xcb::CONFIG_WINDOW_HEIGHT as u16, window_height as u32),
            ],
        );
    }

    fn filtered_commands(&self) -> Vec<&String> {
        self.commands
            .iter()
            .filter(|c| c.starts_with(self.search_input.as_str()))
            .skip(std::cmp::min(self.commands.len() - MAX_ITEMS as usize, self.selection_index))
            .take(5)
            .collect::<Vec<&String>>()
    }

    fn item_height(&self) -> u16 {
        self.config.font_size + self.config.font_size / 2
    }

    fn window_height(&self) -> u16 {
        let total_commands = self.filtered_commands().len();

        self.item_height() * (total_commands + 1) as u16
    }
}
