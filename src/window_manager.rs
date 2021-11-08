use crate::client::{Client};
use crate::errors::{Result};
use crate::key::{KeyPair};
use std::collections::{VecDeque, HashMap};
use std::process::Command;

pub enum Event {
    Forward,
    Backward,
}

pub struct Handler {
    pub command: Option<Box<dyn Fn() -> Command>>,
    pub event: Option<Event>,
}

pub struct WindowManager {
    conn: xcb::Connection,
    clients: VecDeque<Client>,
    active_window: usize,
    keys: HashMap<KeyPair, Handler>,
    running: bool,
}

impl WindowManager {
    pub fn new(keys: HashMap<KeyPair, Handler>) -> Self {
        let (conn, _) = xcb::Connection::connect(None)
            .expect("Unable to access your display. Check your DISPLAY enviroment variable.");

        let screen = match conn.get_setup().roots().next() {
            Some(s) => s,
            None => panic!("Unable to find a screen."),
        };

        let key_symbols = xcb_util::keysyms::KeySymbols::new(&conn);
        for pair in keys.keys() {
            match key_symbols.get_keycode(pair.keysym).next() {
                Some(keycode) => {
                    xcb::grab_key(
                        &conn,
                        false,
                        screen.root(),
                        pair.modifiers,
                        keycode,
                        xcb::GRAB_MODE_ASYNC as u8,
                        xcb::GRAB_MODE_ASYNC as u8
                    );

                },
                _ => {
                    dbg!("Failed to find keycode for keysym: {}", pair.keysym);
                }
            }
        }

        drop(key_symbols);

        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT
        )];

        let cookie = xcb::change_window_attributes_checked(&conn, screen.root(), &values);

        match cookie.request_check() {
            Ok(_) => (),
            Err(_) => panic!(
                "Unable to change window attributes. Is another window manager running?"
            ),
        }

        Self {
            conn: conn,
            active_window: 0,
            clients: VecDeque::new(),
            keys,
            running: false,
        }
    }

    pub fn run(&mut self) {
        self.running = true;

        while self.running {
            let event = match self.conn.wait_for_event() {
                Some(e) => e,
                _ => continue,
            };

            let status = match event.response_type() {
                xcb::KEY_PRESS => self.on_key_press(unsafe { xcb::cast_event(&event) }),
                xcb::CONFIGURE_REQUEST => self.on_configure_request(unsafe { xcb::cast_event(&event) }), 
                xcb::MAP_REQUEST => self.on_map_request(unsafe { xcb::cast_event(&event) }), 
                xcb::ENTER_NOTIFY => self.on_enter_notify(unsafe { xcb::cast_event(&event) }),
                _ => continue,
            };

            if status.is_err() {
                dbg!("Error occured processing event: {:?}", event.response_type());
            }

            self.conn.flush();
        }
    }

    fn on_key_press(&mut self, event: &xcb::KeyPressEvent) -> Result<()> {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&self.conn);
        for pair in self.keys.keys() {
            match key_symbols.get_keycode(pair.keysym).next() {
                Some(keycode) => {
                    if keycode == event.detail() && pair.modifiers == event.state() {
                        let key = self.keys.get(pair).unwrap();
                        if let Some(command) = &key.command {
                            (*command)().spawn().unwrap();
                        }
                    }
                },
                _ => {
                    dbg!("Failed to find keycode for keysym: {}", pair.keysym);
                }
            }
        }

        Ok(())
    }

    fn on_configure_request(&mut self, event: &xcb::ConfigureRequestEvent) -> Result<()> {
        let mut values = Vec::new();

        if event.value_mask() & xcb::CONFIG_WINDOW_X as u16 > 0 {
            values.push((xcb::CONFIG_WINDOW_X as u16, event.x() as u32))
        }

        if event.value_mask() & xcb::CONFIG_WINDOW_Y as u16 > 0 {
            values.push((xcb::CONFIG_WINDOW_Y as u16, event.y() as u32))
        }

        if event.value_mask() & xcb::CONFIG_WINDOW_WIDTH as u16 > 0 {
            values.push((xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32))
        }

        if event.value_mask() & xcb::CONFIG_WINDOW_HEIGHT as u16 > 0 {
            values.push((xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32))
        }

        if event.value_mask() & xcb::CONFIG_WINDOW_BORDER_WIDTH as u16 > 0 {
            values.push((xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, event.border_width() as u32))
        }

        if event.value_mask() & xcb::CONFIG_WINDOW_SIBLING as u16 > 0 {
            values.push((xcb::CONFIG_WINDOW_SIBLING as u16, event.sibling() as u32))
        }

        if event.value_mask() & xcb::CONFIG_WINDOW_STACK_MODE as u16 > 0 {
            values.push((xcb::CONFIG_WINDOW_STACK_MODE as u16, event.stack_mode() as u32))
        }

        xcb::configure_window(&self.conn, event.window(), &values);

        Ok(())
    }

    fn on_map_request(&mut self, event: &xcb::MapRequestEvent) -> Result<()> {
        if self.has_override_redirect(event.window()) {
            return Ok(());
        }

        if let Some(_) = self.window_to_client(event.window()) {
            return Ok(());
        }

        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_ENTER_WINDOW
        )];

        xcb::change_window_attributes(&self.conn, event.window(), &values);

        let client = Client {
            window: event.window(),
        };

        self.set_active_window(client.window)?;
        self.clients.push_front(client);

        xcb::map_window(&self.conn, event.window());

        self.resize();

        Ok(())
    }

    fn on_enter_notify(&mut self, event: &xcb::EnterNotifyEvent) -> Result<()> {
        self.set_active_window(event.event())?;

        xcb::set_input_focus(&self.conn, xcb::INPUT_FOCUS_PARENT as u8, event.event(), xcb::CURRENT_TIME);

        Ok(())
    }

    fn has_override_redirect(&self, window: xcb::Window) -> bool {
        let cookie = xcb::get_window_attributes(&self.conn, window);

        if let Ok(attrs) = cookie.get_reply() {
            attrs.override_redirect() 
        } else {
            false
        }
    }

    fn window_to_client(&self, window: xcb::Window) -> Option<&Client> {
        for client in &self.clients {
            if client.window == window {
                return Some(&client);
            }
        }

        None
    }

    fn get_screen(&self) -> xcb::Screen {
        return match self.conn.get_setup().roots().next() {
            Some(s) => s,
            None => panic!("Unable to find a screen."),
        };
    }

    fn resize(&mut self) {
        let screen = self.get_screen();

        let border = 4;
        let border_double = border * 2;
        let gap = 6;
        let gap_double = gap * 2;
        let screen_width = screen.width_in_pixels() as usize;
        let screen_height = screen.height_in_pixels() as usize;
        let clients_length = self.clients.len();

        for (i, client) in self.clients.iter().enumerate() {
            let (mut x, mut y) = (gap, gap);

            let (mut width, mut height) = (
                screen_width - border_double - gap_double,
                screen_height - border_double - gap_double,
            );

            if clients_length > 1 {
                width = (width - border_double - gap_double) / 2;

                if i > 0 {
                    let window_height = screen_height / (clients_length - 1);

                    x = width + border_double + gap_double + gap;
                    y = window_height * (i - 1) + gap;

                    height = window_height - border_double - gap_double;
                }
            }

            xcb::configure_window(&self.conn, client.window, &[
                (xcb::CONFIG_WINDOW_X as u16, x as u32),
                (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                (xcb::CONFIG_WINDOW_WIDTH as u16, width as u32),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, height as u32),
                (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border as u32),
            ]);
        }
    }

    fn set_active_window(&mut self, window: xcb::Window) -> Result<()> {
        xcb::change_window_attributes(&self.conn, window, &[
            (xcb::CW_BORDER_PIXEL, 0x3b7a82),
        ]);

        for (i, client) in self.clients.iter().enumerate() {
            if client.window == window {
                self.active_window = i;
            } else {
                xcb::change_window_attributes(&self.conn, client.window, &[
                    (xcb::CW_BORDER_PIXEL, 0x444444),
                ]);
            }
        }

        Ok(())
    }
}
