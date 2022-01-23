use crate::client::Client;
use crate::config::Config;
use std::collections::VecDeque;

pub struct InitContext<'a> {
    pub conn: &'a xcb::Connection,
    pub screen: &'a xcb::Screen<'a>,
}

pub struct EventContext<'a, E> {
    pub conn: &'a xcb::Connection,
    pub clients: &'a VecDeque<Client>,
    pub screen: &'a xcb::Screen<'a>,
    pub config: &'a Config,
    pub event: E,
}

pub type KeyPressContext<'a> = EventContext<'a, &'a xcb::KeyPressEvent>;
pub type ConfigureRequestContext<'a> = EventContext<'a, &'a xcb::ConfigureRequestEvent>;
pub type MapRequestContext<'a> = EventContext<'a, &'a xcb::MapRequestEvent>;
pub type EnterNotifyContext<'a> = EventContext<'a, &'a xcb::EnterNotifyEvent>;
pub type UnmapNotifyContext<'a> = EventContext<'a, &'a xcb::UnmapNotifyEvent>;

pub trait PluginHandler {
    fn init(&self, _: InitContext) {}
    fn on_key_press(&mut self, _: KeyPressContext) {}
    fn on_configure_request(&mut self, _: ConfigureRequestContext) {}
    fn on_map_request(&mut self, _: MapRequestContext) {}
    fn on_enter_notify(&mut self, _: EnterNotifyContext) {}
    fn on_unmap_notify(&mut self, _: UnmapNotifyContext) {}
}

pub struct Plugin<C> {
    pub context: C,
}
