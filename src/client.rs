use crate::config::{Action, Config};
use crate::screen::get_screen;
use std::sync::Arc;
use actix::{Actor, Context, Handler, Message};

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
    pub clients: Vec<Client>,
    pub active_workspace: u8,
    pub active_window: Option<xcb::Window>,
}

impl Clients {
    pub fn new(conn: Arc<xcb_util::ewmh::Connection>) -> Self {
        Self {
            conn,
            clients: vec![],
            active_workspace: 1,
            active_window: None,
        }
    }

    fn set_client_list(&mut self) {
        xcb_util::ewmh::set_client_list(
            &self.conn,
            0,
            &self.clients.iter().map(|c| c.window).collect::<Vec<u32>>()
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

    fn handle(&mut self, msg: CreateClient, _ctx: &mut Self::Context) -> Self::Result {
        let cookie = xcb_util::ewmh::get_wm_window_type(
            &self.conn,
            msg.window
        ).get_reply();

        let mut controlled = true;

        if let Ok(reply) = cookie {
            let atoms = reply.atoms();
            for atom in atoms {
                if *atom == self.conn.WM_WINDOW_TYPE_DOCK() {
                    controlled = false;
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
        }

        // There won't be many clients, so this isn't completely horrible.
        // Vec is easier for actors to handle compared to VecDeque
        // because MessageResponse is implemented for Vec.
        self.clients.insert(0, Client {
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

pub struct GetClients;

impl Message for GetClients {
    type Result = Vec<Client>;
}

impl Handler<GetClients> for Clients {
    type Result = Vec<Client>;

    fn handle(&mut self, _msg: GetClients, _ctx: &mut Self::Context) -> Self::Result {
        self.clients.clone()
    }
}

pub struct ResizeClients {
    pub config: Arc<Config>,
}

impl Message for ResizeClients {
    type Result = ();
}

impl Handler<ResizeClients> for Clients {
    type Result = ();

    fn handle(&mut self, msg: ResizeClients, _ctx: &mut Self::Context) -> Self::Result {
        let screen = get_screen(&self.conn);

        let screen_width = screen.width_in_pixels() as usize;
        let screen_height = screen.height_in_pixels() as usize;
        let border = msg.config.border_thickness as usize;
        let border_double = border * 2;
        let gap = msg.config.border_gap as usize;
        let gap_double = gap * 2;

        let visible_clients = self.clients
            .iter()
            .filter(|&c| c.visible && c.controlled)
            .cloned()
            .collect::<Vec<Client>>();

        let padding_top = self.clients.iter()
            .filter(|&c| c.visible)
            .fold(0, |acc, c| acc + c.padding_top) as usize;

        let clients_length = visible_clients.len();
        let available_height = screen_height - padding_top;

        for (i, client) in visible_clients.iter().enumerate() {
            let (mut x, mut y) = (gap, gap + padding_top);

            let (mut width, mut height) = (
                screen_width - border_double - gap_double,
                available_height - border_double - gap_double,
            );

            if clients_length > 1 {
                width = (width - border_double - gap_double) / 2;

                if i > 0 {
                    let window_height = (available_height) / (clients_length - 1);

                    x = x + width + border_double + gap_double;
                    y = y + window_height * (i - 1);

                    height = window_height - border_double - gap_double;
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

    fn handle(&mut self, msg: HideWindow, _ctx: &mut Self::Context) -> Self::Result {
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

    fn handle(&mut self, msg: SetActiveWorkspace, _ctx: &mut Self::Context) -> Self::Result {
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

        self.conn.flush();
    }
}

pub struct SetActiveWindow {
    pub config: Arc<Config>,
    pub window: Option<xcb::Window>,
}

impl Message for SetActiveWindow {
    type Result = ();
}

impl Handler<SetActiveWindow> for Clients {
    type Result = ();

    fn handle(&mut self, msg: SetActiveWindow, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(window) = msg.window {
            let active_border = msg.config.active_border;
            let inactive_border = msg.config.inactive_border;

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

        self.conn.flush();
    }
}

pub struct HandleWindowAction {
    pub config: Arc<Config>,
    pub action: Action,
    pub window: xcb::Window,
}

impl Message for HandleWindowAction {
    type Result = ();
}

impl Handler<HandleWindowAction> for Clients {
    type Result = ();

    fn handle(&mut self, msg: HandleWindowAction, _ctx: &mut Self::Context) -> Self::Result {
        let pos = self.clients
            .iter()
            .position(|c| Some(c.window) == self.active_window)
            .unwrap_or(0);

        let new_active_window = match msg.action {
            Action::SelectLeftWindow => {
                if pos >= self.clients.len() - 1 {
                    0
                } else {
                    pos + 1
                }
            },
            Action::SelectRightWindow => {
                if pos == 0 && self.clients.len() == 0 {
                    0
                } else if pos == 0 && self.clients.len() > 0 {
                    self.clients.len() - 1
                } else {
                    pos - 1
                }
            },
        };

        self.active_window = Some(new_active_window as u32);
    }
}
