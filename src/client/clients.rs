use crate::client::Client;
use crate::config::Config;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

pub struct Clients {
    pub conn: Arc<xcb_util::ewmh::Connection>,
    pub config: Arc<Config>,
    pub clients: VecDeque<Client>,
    pub active_workspace: u8,
    pub active_window: HashMap<u8, Option<xcb::Window>>,
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
            active_window: HashMap::new(),
            dock_window: None,
            front_window_ratio: HashMap::new(),
        }
    }

    pub fn refresh_clients(&mut self) {
        xcb_util::ewmh::set_client_list(
            &self.conn,
            0,
            &self.clients.iter().map(|c| c.window).collect::<Vec<u32>>(),
        );

        self.set_workspace_names();
    }
}
