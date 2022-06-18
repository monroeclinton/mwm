use crate::event::EventContext;
use anyhow::Result;

pub trait PluginHandler {
    fn on_client_message(&mut self, _ectx: EventContext<xcb::ClientMessageEvent>) -> Result<()> { Ok(()) }
    fn on_key_press(&mut self, _ectx: EventContext<xcb::KeyPressEvent>) -> Result<()> { Ok(()) }
    fn on_configure_request(&mut self, _ectx: EventContext<xcb::ConfigureRequestEvent>) -> Result<()> { Ok(()) }
    fn on_map_request(&mut self, _ectx: EventContext<xcb::MapRequestEvent>) -> Result<()> { Ok(()) }
    fn on_property_notify(&mut self, _ectx: EventContext<xcb::PropertyNotifyEvent>) -> Result<()> { Ok(()) }
    fn on_enter_notify(&mut self, _ectx: EventContext<xcb::EnterNotifyEvent>) -> Result<()> { Ok(()) }
    fn on_unmap_notify(&mut self, _ectx: EventContext<xcb::UnmapNotifyEvent>) -> Result<()> { Ok(()) }
    fn on_destroy_notify(&mut self, _ectx: EventContext<xcb::DestroyNotifyEvent>) -> Result<()> { Ok(()) }
}
