use crate::state::State;
use smithay::reexports::wayland_server::{backend, Display};

pub struct Data {
    pub display: Display<State>,
    pub state: State,
}

pub struct ClientData;

impl backend::ClientData for ClientData {}
