use crate::config::{Action, Config};
use crate::screen::get_screen;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

#[derive(Clone, Eq, PartialEq)]
pub struct Client {
    pub window: xcb::Window,
    pub workspace: Option<u8>,
    pub visible: bool,
    pub controlled: bool, // If should resize/size/configure window
    pub full_screen: bool,
    pub padding_top: u32,
}

pub struct Clients {
    pub conn: Arc<xcb_util::ewmh::Connection>,
    pub config: Arc<Config>,
    pub clients: VecDeque<Client>,
    pub active_workspace: u8,
    pub active_window: Option<xcb::Window>,
    pub dock_window: Option<xcb::Window>,
    pub front_window_ratio: HashMap<u8, f32>,
}

impl Clients {
    pub fn new(conn: Arc<xcb_util::ewmh::Connection>, config: Arc<Config>) -> Self {
        Self {
            conn,
            config,
            clients: VecDeque::new(),
            active_workspace: 1,
            active_window: None,
            dock_window: None,
            front_window_ratio: HashMap::new(),
        }
    }

    pub fn create(&mut self, window: xcb::Window) {
        let already_created = self.clients.iter().any(|c| c.window == window);

        if already_created {
            return;
        }

        let reply = xcb_util::ewmh::get_wm_window_type(&self.conn, window).get_reply();

        let mut controlled = true;

        if let Ok(window_type) = reply {
            let atoms = window_type.atoms();
            for atom in atoms {
                if *atom == self.conn.WM_WINDOW_TYPE_DOCK() {
                    self.dock_window = Some(window);
                    controlled = false;
                }

                if *atom == self.conn.WM_WINDOW_TYPE_DIALOG() {
                    controlled = false;
                }
            }
        }

        let cookie = xcb_util::ewmh::get_wm_strut_partial(&self.conn, window).get_reply();

        // TODO: Add other paddings
        let padding_top = if let Ok(struct_partial) = cookie {
            struct_partial.top
        } else {
            0
        };

        let workspace = if controlled {
            Some(self.active_workspace)
        } else {
            None
        };

        self.clients.push_front(Client {
            window,
            workspace,
            visible: true,
            controlled,
            full_screen: false,
            padding_top,
        });

        // Ensure border width and color is set for non-dock windows
        if self.dock_window != Some(window) {
            xcb::configure_window(
                &self.conn,
                window,
                &[(
                    xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
                    self.config.border_thickness,
                )],
            );

            xcb::change_window_attributes(
                &self.conn,
                window,
                &[(xcb::CW_BORDER_PIXEL, self.config.inactive_border)],
            );

            // Set window as active
            self.set_active_window(Some(window));
        }

        // Make sure window does not overlap with statusbar
        if controlled {
            xcb::configure_window(
                &self.conn,
                window,
                &[(xcb::CONFIG_WINDOW_Y as u16, self.get_padding_top() as u32)],
            );
        }

        xcb::map_window(&self.conn, window);

        self.conn.flush();

        self.set_client_list();
    }

    pub fn destroy(&mut self, window: xcb::Window) {
        self.clients.retain(|c| c.window != window);

        if self.active_window == Some(window) {
            let active_window = self
                .clients
                .iter()
                .filter(|c| c.controlled)
                .next()
                .map_or(None, |c| Some(c.window));
            self.set_active_window(active_window);
        }

        self.set_client_list();
    }

    pub fn hide(&mut self, window: xcb::Window) {
        for mut client in self.clients.iter_mut() {
            if window == client.window {
                if client.visible {
                    xcb::unmap_window(&self.conn, client.window);
                }

                client.visible = false;
                break;
            }
        }

        self.conn.flush();

        self.resize();
    }

    pub fn resize(&mut self) {
        let screen = get_screen(&self.conn);

        let screen_width = screen.width_in_pixels() as usize;
        let screen_height = screen.height_in_pixels() as usize;
        let border = self.config.border_thickness as usize;
        let border_double = border * 2;
        let gap = self.config.border_gap as usize;
        let gap_double = gap * 2;

        let padding_top = self.get_padding_top();

        let max_clients = (screen_height - padding_top) / (gap_double + border_double) - 1;

        let visible_clients = self
            .clients
            .iter()
            .filter(|&c| c.visible && c.controlled)
            .take(max_clients)
            .cloned()
            .collect::<Vec<Client>>();

        let clients_length = visible_clients.len();
        let available_height = screen_height - padding_top;

        let front_window_ratio = *self
            .front_window_ratio
            .entry(self.active_workspace)
            .or_insert(0.5);

        // Tile windows
        for (i, client) in visible_clients.iter().enumerate() {
            let (mut x, mut y) = (gap, gap + padding_top);

            let (mut width, mut height) = (
                screen_width - border_double - gap_double,
                available_height - border_double - gap_double,
            );

            if clients_length > 1 {
                width -= border_double - gap_double;

                let front_window_width = (width as f32 * front_window_ratio) as usize;
                let window_height = (available_height) / (clients_length - 1);

                if i > 0 {
                    width -= front_window_width;
                    height = window_height - border_double - gap_double;

                    x += front_window_width + border_double + gap_double;
                    y += window_height * (i - 1);
                } else {
                    width = front_window_width;
                }
            }

            xcb::configure_window(
                &self.conn,
                client.window,
                &[
                    (xcb::CONFIG_WINDOW_X as u16, x as u32),
                    (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, width as u32),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, height as u32),
                    (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border as u32),
                ],
            );
        }

        // Full screen windows
        for client in visible_clients.iter().filter(|&c| c.full_screen) {
            xcb::configure_window(
                &self.conn,
                client.window,
                &[
                    (xcb::CONFIG_WINDOW_X as u16, 0),
                    (xcb::CONFIG_WINDOW_Y as u16, 0),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, screen_width as u32),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, screen_height as u32),
                    (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, 0),
                ],
            );
        }

        self.conn.flush();
    }

    pub fn set_controlled_status(&mut self, window: xcb::Window, status: bool) {
        for mut client in self.clients.iter_mut() {
            if window == client.window {
                client.controlled = status;
                break;
            }
        }
    }

    pub fn set_active_workspace(&mut self, workspace: u8) {
        self.active_workspace = workspace;

        for mut client in self.clients.iter_mut().filter(|c| c.controlled) {
            if Some(self.active_workspace) == client.workspace {
                if !client.visible {
                    xcb::map_window(&self.conn, client.window);
                }

                client.visible = true;
            } else {
                if client.visible {
                    xcb::unmap_window(&self.conn, client.window);
                }

                client.visible = false;
            }
        }

        xcb_util::ewmh::set_current_desktop(&self.conn, 0, self.active_workspace as u32);

        self.set_active_window(None);

        self.resize();

        self.conn.flush();
    }

    pub fn set_active_window(&mut self, window: Option<xcb::Window>) {
        if window == self.dock_window {
            return;
        }

        let active_border = self.config.active_border;
        let inactive_border = self.config.inactive_border;

        if let Some(window) = window {
            xcb::set_input_focus(
                &self.conn,
                xcb::INPUT_FOCUS_PARENT as u8,
                window,
                xcb::CURRENT_TIME,
            );
            xcb::change_window_attributes(
                &self.conn,
                window,
                &[(xcb::CW_BORDER_PIXEL, active_border)],
            );
        }

        xcb_util::ewmh::set_active_window(&self.conn, 0, window.unwrap_or(xcb::WINDOW_NONE));

        if window != self.active_window {
            if let Some(active_window) = self.active_window {
                xcb::change_window_attributes(
                    &self.conn,
                    active_window,
                    &[(xcb::CW_BORDER_PIXEL, inactive_border)],
                );
            }

            self.active_window = window;
        }

        self.conn.flush();
    }

    pub fn set_window_workspace(&mut self, window: xcb::Window, workspace: Option<u8>) {
        if Some(self.active_workspace) == workspace {
            return;
        }

        for client in self.clients.iter_mut() {
            if client.window == window {
                client.workspace = workspace;

                self.hide(window);

                break;
            }
        }

        self.set_client_list();
    }

    pub fn set_full_screen(&mut self, window: xcb::Window, status: Option<bool>, toggle: bool) {
        for mut client in self.clients.iter_mut() {
            if window == client.window {
                client.full_screen = Some(true) == status || (!client.full_screen && toggle);
                self.resize();
                break;
            }
        }
    }

    pub fn handle_action(&mut self, _window: xcb::Window, action: Action) {
        // Handle close action
        if let (Action::Close, Some(window)) = (&action, self.active_window) {
            let delete_window = xcb::intern_atom(&self.conn, false, "WM_DELETE_WINDOW")
                .get_reply()
                .unwrap();

            let reply =
                xcb_util::icccm::get_wm_protocols(&self.conn, window, self.conn.WM_PROTOCOLS())
                    .get_reply();

            let supports_wm_delete_window = reply.unwrap().atoms().contains(&delete_window.atom());

            if supports_wm_delete_window {
                let event = xcb::ClientMessageEvent::new(
                    32,
                    window,
                    self.conn.WM_PROTOCOLS(),
                    xcb::ClientMessageData::from_data32([
                        delete_window.atom(),
                        xcb::CURRENT_TIME,
                        0,
                        0,
                        0,
                    ]),
                );

                xcb::send_event_checked(&self.conn, true, window, xcb::EVENT_MASK_NO_EVENT, &event);
            } else {
                xcb::set_close_down_mode(&self.conn, xcb::CLOSE_DOWN_DESTROY_ALL as u8);
                xcb::kill_client(&self.conn, window);
                self.conn.flush();
            }
        }

        let clients = self
            .clients
            .iter()
            .filter(|&c| c.visible && c.controlled)
            .cloned()
            .collect::<Vec<Client>>();

        let pos = clients
            .iter()
            .position(|c| Some(c.window) == self.active_window)
            .unwrap_or(0);

        // Handle the selection actions
        let new_pos = match action {
            Action::SelectAbove => {
                if clients.len() <= 1 {
                    0
                } else if pos == 0 && !clients.is_empty() {
                    clients.len() - 1
                } else {
                    pos - 1
                }
            }
            Action::SelectBelow => {
                if clients.len() <= 1 || pos >= clients.len() - 1 {
                    0
                } else {
                    pos + 1
                }
            }
            _ => pos,
        };

        let active_client = clients.get(new_pos);
        if let Some(client) = active_client {
            self.set_active_window(Some(client.window));
        }

        // Handle the window sizing actions
        let size = self
            .front_window_ratio
            .entry(self.active_workspace)
            .or_insert(0.5);

        match action {
            Action::ShrinkFront => {
                if *size > 0.10 {
                    *size -= 0.05;
                }
            }
            Action::ExpandFront => {
                if *size < 0.9 {
                    *size += 0.05;
                }
            }
            _ => (),
        };

        self.resize();
    }

    fn get_padding_top(&self) -> usize {
        self.clients
            .iter()
            .filter(|&c| c.visible)
            .fold(0, |acc, c| acc + c.padding_top as usize)
    }

    fn set_client_list(&mut self) {
        xcb_util::ewmh::set_client_list(
            &self.conn,
            0,
            &self.clients.iter().map(|c| c.window).collect::<Vec<u32>>(),
        );

        let names = (1..=9)
            .map(|i: u8| {
                let count = self
                    .clients
                    .iter()
                    .filter(|c| c.workspace == Some(i))
                    .count();

                let count_string = count
                    .to_string()
                    .replace('0', "⁰")
                    .replace('1', "¹")
                    .replace('2', "²")
                    .replace('3', "³")
                    .replace('4', "⁴")
                    .replace('5', "⁵")
                    .replace('6', "⁶")
                    .replace('7', "⁷")
                    .replace('8', "⁸")
                    .replace('9', "⁹");

                if count > 0 {
                    format!("{}{}", i, count_string)
                } else {
                    i.to_string()
                }
            })
            .collect::<Vec<String>>();

        xcb_util::ewmh::set_desktop_names(&self.conn, 0, names.iter().map(|s| s.as_ref()));
    }
}
