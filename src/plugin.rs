use crate::client::Client;
use crate::config::Config;
use std::collections::VecDeque;

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
    fn on_key_press(&self, _: KeyPressContext) {}
    fn on_configure_request(&self, _: ConfigureRequestContext) {}
    fn on_map_request(&self, _: MapRequestContext) {}
    fn on_enter_notify(&self, _: EnterNotifyContext) {}
    fn on_unmap_notify(&self, _: UnmapNotifyContext) {}
}

pub struct Plugin<C> {
    pub context: C,
}
