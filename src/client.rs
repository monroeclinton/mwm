use crate::config::{Action, Config};
use crate::screen::get_screen;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use actix::{Actor, Context, Handler, Message, AsyncContext};

#[derive(Clone, PartialEq)]
pub struct Client {
    pub window: xcb::Window,
    pub workspace: Option<u8>,
    pub visible: bool,
    pub controlled: bool, // If should resize/size/configure window
    pub padding_top: u32,
}

pub struct Clients {
    pub conn: Arc<xcb_util::ewmh::Connection>,
    pub config: Arc<Config>,
    pub clients: VecDeque<Client>,
    pub active_workspace: u8,
    pub active_window: Option<xcb::Window>,
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
            front_window_ratio: HashMap::new(),
        }
    }

    fn set_client_list(&mut self) {
        xcb_util::ewmh::set_client_list(
            &self.conn,
            0,
            &self.clients.iter().map(|c| c.window).collect::<Vec<u32>>()
        );

        let names = (1..=9)
            .map(|i: u8| {
                let count = self.clients.iter()
                    .filter(|c| c.workspace == Some(i))
                    .count();

                let count_string = count.to_string()
                    .replace("0", "⁰")
                    .replace("1", "¹")
                    .replace("2", "²")
                    .replace("3", "³")
                    .replace("4", "⁴")
                    .replace("5", "⁵")
                    .replace("6", "⁶")
                    .replace("7", "⁷")
                    .replace("8", "⁸")
                    .replace("9", "⁹");

                if count > 0 {
                    format!("{}{}", i.to_string(), count_string)
                } else {
                    i.to_string()
                }
            })
            .collect::<Vec<String>>();

        xcb_util::ewmh::set_desktop_names(
            &self.conn,
            0,
            names.iter().map(|s| s.as_ref()),
        );
    }
}

impl Actor for Clients {
    type Context = Context<Self>;
}

pub struct CreateClient {
    pub window: xcb::Window,
}

impl Message for CreateClient {
    type Result = ();
}

impl Handler<CreateClient> for Clients {
    type Result = ();

    fn handle(&mut self, msg: CreateClient, ctx: &mut Self::Context) -> Self::Result {
        let already_created = self.clients.iter().any(|c| c.window == msg.window);

        if already_created {
            return;
        }

        let reply = xcb::get_property(
            &self.conn,
            false,
            msg.window,
            xcb::ATOM_WM_TRANSIENT_FOR,
            xcb::ATOM_WINDOW,
            0,
            1,
        ).get_reply();

        let is_transient = match reply {
            Ok(property) => {
                property.format() == 32 ||
                property.type_() == xcb::ATOM_WINDOW ||
                property.value_len() != 0
            },
            _ => false,
        };

        let reply = xcb_util::ewmh::get_wm_window_type(
            &self.conn,
            msg.window
        ).get_reply();

        let mut controlled = true;

        if let Ok(window_type) = reply {
            let atoms = window_type.atoms();
            for atom in atoms {
                if *atom == self.conn.WM_WINDOW_TYPE_DOCK() {
                    controlled = false;
                }

                if *atom == self.conn.WM_WINDOW_TYPE_DIALOG() || is_transient {
                    controlled = false;

                    let screen = get_screen(&self.conn);

                    let reply = xcb::get_geometry(&self.conn, msg.window).get_reply();

                    if let Ok(geomtry) = reply {
                        let x = (screen.width_in_pixels() - geomtry.width()) / 2;
                        let y = (screen.height_in_pixels() - geomtry.height()) / 2;
                        let border = self.config.border_thickness;

                        xcb::configure_window(
                            &self.conn,
                            msg.window,
                            &[
                                (xcb::CONFIG_WINDOW_X as u16, x as u32),
                                (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                                (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border as u32),
                            ],
                        );
                    }
                }
            }
        }

        let cookie = xcb_util::ewmh::get_wm_strut_partial(
            &self.conn,
            msg.window
        ).get_reply();

        // TODO: Add other paddings
        let mut padding_top = 0;
        if let Ok(struct_partial) = cookie {
            padding_top = struct_partial.top;
        }

        let mut workspace = None;
        if controlled {
            workspace = Some(self.active_workspace);

            ctx.notify(SetActiveWindow {
                window: Some(msg.window),
            });
        }

        self.clients.push_front(Client {
            window: msg.window,
            workspace,
            visible: true,
            controlled,
            padding_top,
        });

        self.set_client_list();
    }
}

pub struct DestroyClient {
    pub window: xcb::Window,
}

impl Message for DestroyClient {
    type Result = ();
}

impl Handler<DestroyClient> for Clients {
    type Result = ();

    fn handle(&mut self, msg: DestroyClient, _ctx: &mut Self::Context) -> Self::Result {
        self.clients.retain(|c| c.window != msg.window);

        self.set_client_list();
    }
}

pub struct ResizeClients;

impl Message for ResizeClients {
    type Result = ();
}

impl Handler<ResizeClients> for Clients {
    type Result = ();

    fn handle(&mut self, _msg: ResizeClients, _ctx: &mut Self::Context) -> Self::Result {
        let screen = get_screen(&self.conn);

        let screen_width = screen.width_in_pixels() as usize;
        let screen_height = screen.height_in_pixels() as usize;
        let border = self.config.border_thickness as usize;
        let border_double = border * 2;
        let gap = self.config.border_gap as usize;
        let gap_double = gap * 2;

        let padding_top = self.clients.iter()
            .filter(|&c| c.visible)
            .fold(0, |acc, c| acc + c.padding_top) as usize;

        let max_clients = (screen_height - padding_top) / (
            gap_double + border_double
        ) - 1;

        let visible_clients = self.clients
            .iter()
            .filter(|&c| c.visible && c.controlled)
            .take(max_clients)
            .cloned()
            .collect::<Vec<Client>>();

        let clients_length = visible_clients.len();
        let available_height = screen_height - padding_top;

        let front_window_ratio = *self.front_window_ratio
            .entry(self.active_workspace)
            .or_insert(0.5);

        for (i, client) in visible_clients.iter().enumerate() {
            let (mut x, mut y) = (gap, gap + padding_top);

            let (mut width, mut height) = (
                screen_width - border_double - gap_double,
                available_height - border_double - gap_double,
            );

            if clients_length > 1 {
                width = width - border_double - gap_double;

                let front_window_width = (width as f32 * front_window_ratio) as usize;
                let window_height = (available_height) / (clients_length - 1);

                if i > 0 {
                    width = width - front_window_width;
                    height = window_height - border_double - gap_double;

                    x = x + front_window_width + border_double + gap_double;
                    y = y + window_height * (i - 1);
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

        self.conn.flush();
    }
}

pub struct HideWindow {
    pub window: xcb::Window,
}

impl Message for HideWindow {
    type Result = ();
}

impl Handler<HideWindow> for Clients {
    type Result = ();

    fn handle(&mut self, msg: HideWindow, ctx: &mut Self::Context) -> Self::Result {
        for mut client in self.clients.iter_mut() {
            if msg.window == client.window {
                if client.visible {
                    xcb::unmap_window(&self.conn, client.window);
                }

                client.visible = false;
                break;
            }
        }

        self.conn.flush();

        ctx.notify(ResizeClients);
    }
}

pub struct SetControlledStatus {
    pub window: xcb::Window,
    pub status: bool,
}

impl Message for SetControlledStatus {
    type Result = ();
}

impl Handler<SetControlledStatus> for Clients {
    type Result = ();

    fn handle(&mut self, msg: SetControlledStatus, _ctx: &mut Self::Context) -> Self::Result {
        for mut client in self.clients.iter_mut() {
            if msg.window == client.window {
                client.controlled = msg.status;
                break;
            }
        }
    }
}

pub struct SetActiveWorkspace {
    pub workspace: u8,
}

impl Message for SetActiveWorkspace {
    type Result = ();
}

impl Handler<SetActiveWorkspace> for Clients {
    type Result = ();

    fn handle(&mut self, msg: SetActiveWorkspace, ctx: &mut Self::Context) -> Self::Result {
        self.active_workspace = msg.workspace;

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

        xcb_util::ewmh::set_current_desktop(
            &self.conn,
            0,
            self.active_workspace as u32,
        );

        ctx.notify(SetActiveWindow {
            window: None,
        });

        ctx.notify(ResizeClients);

        self.conn.flush();
    }
}

pub struct SetActiveWindow {
    pub window: Option<xcb::Window>,
}

impl Message for SetActiveWindow {
    type Result = ();
}

impl Handler<SetActiveWindow> for Clients {
    type Result = ();

    fn handle(&mut self, msg: SetActiveWindow, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(window) = msg.window {
            let active_border = self.config.active_border;
            let inactive_border = self.config.inactive_border;

            xcb::set_input_focus(
                &self.conn,
                xcb::INPUT_FOCUS_PARENT as u8,
                window,
                xcb::CURRENT_TIME,
            );

            xcb::change_window_attributes(&self.conn, window, &[(xcb::CW_BORDER_PIXEL, active_border)]);

            for client in self.clients.iter() {
                if client.window != window {
                    xcb::change_window_attributes(
                        &self.conn,
                        client.window,
                        &[(xcb::CW_BORDER_PIXEL, inactive_border)],
                    );
                }
            }

            xcb_util::ewmh::set_active_window(&self.conn, 0, window);
        } else {
            xcb_util::ewmh::set_active_window(&self.conn, 0, 0);
        }

        self.active_window = msg.window;

        self.conn.flush();
    }
}

pub struct SetWindowWorkspace {
    pub window: xcb::Window,
    pub workspace: Option<u8>,
}

impl Message for SetWindowWorkspace {
    type Result = ();
}

impl Handler<SetWindowWorkspace> for Clients {
    type Result = ();

    fn handle(&mut self, msg: SetWindowWorkspace, ctx: &mut Self::Context) -> Self::Result {
        if Some(self.active_workspace) == msg.workspace {
            return;
        }

        for client in self.clients.iter_mut() {
            if client.window == msg.window {
                client.workspace = msg.workspace;

                ctx.notify(HideWindow {
                    window: msg.window,
                });

                break;
            }
        }

        self.set_client_list();
    }
}

pub struct HandleWindowAction {
    pub action: Action,
    pub window: xcb::Window,
}

impl Message for HandleWindowAction {
    type Result = ();
}

impl Handler<HandleWindowAction> for Clients {
    type Result = ();

    fn handle(&mut self, msg: HandleWindowAction, ctx: &mut Self::Context) -> Self::Result {
        // Handle close action
        match msg.action {
            Action::CloseWindow => {
                if let Some(window) = self.active_window {
                    xcb::set_close_down_mode(&self.conn, xcb::CLOSE_DOWN_DESTROY_ALL as u8);
                    xcb::kill_client(&self.conn, window);
                    self.conn.flush();
                }

                return;
            },
            _ => (),
        };

        // Handle the selection actions
        let clients = self.clients
            .iter()
            .filter(|&c| c.visible && c.controlled)
            .cloned()
            .collect::<Vec<Client>>();

        let pos = clients
            .iter()
            .position(|c| Some(c.window) == self.active_window)
            .unwrap_or(0);

        let new_pos = match msg.action {
            Action::SelectAboveWindow => {
                if clients.len() <= 1 {
                    0
                } else if pos == 0 && clients.len() > 0 {
                    clients.len() - 1
                } else {
                    pos - 1
                }
            },
            Action::SelectBelowWindow => {
                if clients.len() <= 1 {
                    0
                } else if pos >= clients.len() - 1 {
                    0
                } else {
                    pos + 1
                }
            },
            _ => pos,
        };

        let active_client = clients.get(new_pos);
        if let Some(client) = active_client {
            ctx.notify(SetActiveWindow {
                window: Some(client.window),
            });
        }

        // Handle the window sizing actions
        let size = self.front_window_ratio
            .entry(self.active_workspace)
            .or_insert(0.5);

        match msg.action {
            Action::ShrinkFrontWindow => {
                if *size > 0.10 {
                    *size -= 0.05;
                }
            },
            Action::ExpandFrontWindow => {
                if *size < 0.9 {
                    *size += 0.05;
                }
            },
            _ => (),
        };

        ctx.notify(ResizeClients);
    }
}
