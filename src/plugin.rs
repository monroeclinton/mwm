pub struct EventContext<'a, E> {
    pub conn: &'a xcb::Connection,
    pub event: E,
}

pub trait PluginHandler {
    fn on_key_press(&self, _: EventContext<&xcb::KeyPressEvent>) {}
    fn on_configure_request(&self, _: EventContext<&xcb::ConfigureRequestEvent>) {}
    fn on_map_request(&self, _: EventContext<&xcb::MapRequestEvent>) {}
    fn on_enter_notify(&self, _: EventContext<&xcb::EnterNotifyEvent>) {}
    fn on_unmap_notify(&self, _: EventContext<&xcb::UnmapNotifyEvent>) {}
}

pub struct Plugin<C> {
    pub context: C,
}
