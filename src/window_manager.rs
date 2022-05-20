use crate::client::Client;
use crate::config::Config;
use crate::plugin::{EventContext, InitContext, PluginHandler};
use std::collections::VecDeque;
use anyhow::Result;

pub struct WindowManager {
    conn: xcb::Connection,
    clients: VecDeque<Client>,
    plugins: Vec<Box<dyn PluginHandler>>,
    config: Config,
    running: bool,
}

impl WindowManager {
    pub fn new(
        config: Config,
        plugins: Vec<Box<dyn PluginHandler>>,
    ) -> Self {
        let (conn, _) = xcb::Connection::connect(None)
            .expect("Unable to access your display. Check your DISPLAY enviroment variable.");

        let screen = match conn.get_setup().roots().next() {
            Some(s) => s,
            None => panic!("Unable to find a screen."),
        };

        let key_symbols = xcb_util::keysyms::KeySymbols::new(&conn);
        for pair in config.commands.keys() {
            match key_symbols.get_keycode(pair.keysym).next() {
                Some(keycode) => {
                    xcb::grab_key(
                        &conn,
                        false,
                        screen.root(),
                        pair.modifiers,
                        keycode,
                        xcb::GRAB_MODE_ASYNC as u8,
                        xcb::GRAB_MODE_ASYNC as u8,
                    );
                }
                _ => {
                    dbg!("Failed to find keycode for keysym: {}", pair.keysym);
                }
            }
        }

        drop(key_symbols);

        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY,
        )];

        let cookie = xcb::change_window_attributes_checked(&conn, screen.root(), &values);

        match cookie.request_check() {
            Ok(_) => (),
            Err(_) => {
                panic!("Unable to change window attributes. Is another window manager running?")
            }
        }

        for plugin in plugins.iter() {
            plugin.init(InitContext {
                conn: &conn,
                screen: &screen,
            });
        }

        Self {
            conn,
            clients: VecDeque::new(),
            plugins,
            config,
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
                xcb::CONFIGURE_REQUEST => {
                    self.on_configure_request(unsafe { xcb::cast_event(&event) })
                }
                xcb::MAP_REQUEST => self.on_map_request(unsafe { xcb::cast_event(&event) }),
                xcb::ENTER_NOTIFY => self.on_enter_notify(unsafe { xcb::cast_event(&event) }),
                xcb::UNMAP_NOTIFY => self.on_unmap_notify(unsafe { xcb::cast_event(&event) }),
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
        for pair in self.config.commands.keys() {
            if let Some(keycode) = key_symbols.get_keycode(pair.keysym).next() {
                if keycode == event.detail() && pair.modifiers == event.state() {
                    let command = self.config.commands.get(pair).unwrap();
                    (*command)().spawn().unwrap();
                }
            }
        }

        let screen = match self.conn.get_setup().roots().next() {
            Some(s) => s,
            None => panic!("Unable to find a screen."),
        };

        for plugin in self.plugins.iter_mut() {
            plugin.on_key_press(EventContext {
                conn: &self.conn,
                clients: &self.clients,
                config: &self.config,
                screen: &screen,
                event,
            })?;
        }

        Ok(())
    }

    fn on_configure_request(&mut self, event: &xcb::ConfigureRequestEvent) -> Result<()> {
        let values = vec![
            (xcb::CONFIG_WINDOW_X as u16, event.x() as u32),
            (xcb::CONFIG_WINDOW_Y as u16, event.y() as u32),
            (xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32),
            (
                xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
                event.border_width() as u32,
            ),
            (xcb::CONFIG_WINDOW_SIBLING as u16, event.sibling() as u32), // Default: NONE
            (
                xcb::CONFIG_WINDOW_STACK_MODE as u16,
                event.stack_mode() as u32,
            ), // Default: STACK_MODE_ABOVE
        ];

        xcb::configure_window(&self.conn, event.window(), &values);

        let screen = match self.conn.get_setup().roots().next() {
            Some(s) => s,
            None => panic!("Unable to find a screen."),
        };

        for plugin in self.plugins.iter_mut() {
            plugin.on_configure_request(EventContext {
                conn: &self.conn,
                clients: &self.clients,
                config: &self.config,
                screen: &screen,
                event,
            })?;
        }

        Ok(())
    }

    fn on_map_request(&mut self, event: &xcb::MapRequestEvent) -> Result<()> {
        if self.has_override_redirect(event.window()) {
            return Ok(());
        }

        if self.clients.iter().any(|c| c.window == event.window()) {
            return Ok(());
        }

        let values = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_ENTER_WINDOW)];

        xcb::change_window_attributes(&self.conn, event.window(), &values);

        let client = Client {
            window: event.window(),
            visible: true,
        };

        self.clients.push_front(client);

        xcb::map_window(&self.conn, event.window());

        let screen = match self.conn.get_setup().roots().next() {
            Some(s) => s,
            None => panic!("Unable to find a screen."),
        };

        for plugin in self.plugins.iter_mut() {
            plugin.on_map_request(EventContext {
                conn: &self.conn,
                clients: &self.clients,
                config: &self.config,
                screen: &screen,
                event,
            })?;
        }

        Ok(())
    }

    fn on_enter_notify(&mut self, event: &xcb::EnterNotifyEvent) -> Result<()> {
        let screen = match self.conn.get_setup().roots().next() {
            Some(s) => s,
            None => panic!("Unable to find a screen."),
        };

        for plugin in self.plugins.iter_mut() {
            plugin.on_enter_notify(EventContext {
                conn: &self.conn,
                clients: &self.clients,
                config: &self.config,
                screen: &screen,
                event,
            })?;
        }

        Ok(())
    }

    fn on_unmap_notify(&mut self, event: &xcb::UnmapNotifyEvent) -> Result<()> {
        for client in self.clients.iter_mut() {
            if client.window == event.window() {
                client.visible = false;
            }
        }

        let screen = match self.conn.get_setup().roots().next() {
            Some(s) => s,
            None => panic!("Unable to find a screen."),
        };

        for plugin in self.plugins.iter_mut() {
            plugin.on_unmap_notify(EventContext {
                conn: &self.conn,
                clients: &self.clients,
                config: &self.config,
                screen: &screen,
                event,
            })?;
        }

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

    fn remove_window(&mut self, window: xcb::Window) -> Result<()> {
        self.clients.retain(|client| client.window != window);

        Ok(())
    }
}
