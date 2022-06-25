use crate::config::{Config, get_config};
use crate::draw::Draw;
use std::rc::Rc;
use anyhow::Result;

pub struct Launcher {
    conn: Rc<xcb::Connection>,
    config: Rc<Config>,
    draw: Draw,
}

impl Launcher {
    pub fn new() -> Self {
        let (conn, _) = xcb::Connection::connect(None)
            .expect("Unable to access your display. Check your DISPLAY enviroment variable.");

        let config = get_config();

        let screen = conn.get_setup().roots().next()
            .expect("Unable to find a screen.");

        let window = conn.generate_id();

        let x = (screen.width_in_pixels() - config.width) / 2;
        let y = screen.height_in_pixels() / 2;

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
                (xcb::CW_BACK_PIXEL, config.background_color),
                (xcb::CW_OVERRIDE_REDIRECT, 1),
                (xcb::CW_BORDER_PIXEL, config.border_color),
            ],
        );

        xcb::map_window(&conn, window);

        xcb::configure_window(&conn, window, &[
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

        let conn = Rc::new(conn);
        let config = Rc::new(config);
        let draw = Draw::new(conn.clone(), config.clone(), window);

        Self {
            conn,
            config,
            draw,
        }
    }

    pub fn run(mut self) {
        self.draw.draw();

        loop {
            let event = match self.conn.wait_for_event() {
                Some(e) => e,
                _ => continue,
            };

            let status = match event.response_type() {
                xcb::KEY_PRESS => self.on_key_press(unsafe { xcb::cast_event(&event) }),
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

        if let Some(keycode) = key_symbols.get_keycode(self.config.up_keysym).next() {
            if event.detail() == keycode {
                self.draw.move_up()
            }
        }

        if let Some(keycode) = key_symbols.get_keycode(self.config.down_keysym).next() {
            if event.detail() == keycode {
                self.draw.move_down()
            }
        }

        Ok(())
    }
}
