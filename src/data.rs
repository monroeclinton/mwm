use crate::state::State;
use smithay::reexports::wayland_server::{backend, Display};

// Used for the calloop::EventLoop data
pub struct Data {
    pub display: Display<State>,
    pub state: State,
}

// Used to store client data associated with Wayland clients
pub struct ClientData;

impl backend::ClientData for ClientData {}
