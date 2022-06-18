use crate::plugins;
use crate::event::EventContext;
use actix::SystemService;

pub fn on_client_message(context: EventContext<xcb::ClientMessageEvent>) {
    plugins::WindowSelector::from_registry().do_send(context.clone());
    plugins::Workspaces::from_registry().do_send(context.clone());
}

pub fn on_key_press(context: EventContext<xcb::KeyPressEvent>) {
    plugins::Commands::from_registry().do_send(context.clone());
    plugins::WindowSelector::from_registry().do_send(context.clone());
    plugins::Workspaces::from_registry().do_send(context.clone());
}

pub fn on_configure_request(context: EventContext<xcb::ConfigureRequestEvent>) {
    plugins::ConfigureWindow::from_registry().do_send(context);
}

pub fn on_map_request(context: EventContext<xcb::MapRequestEvent>) {
    plugins::MapWindow::from_registry().do_send(context.clone());
    plugins::WindowSizer::from_registry().do_send(context.clone());
}

pub fn on_property_notify(context: EventContext<xcb::PropertyNotifyEvent>) {
    plugins::WindowSizer::from_registry().do_send(context.clone());
    plugins::ConfigureWindow::from_registry().do_send(context);
}

pub fn on_enter_notify(context: EventContext<xcb::EnterNotifyEvent>) {
    plugins::WindowSelector::from_registry().do_send(context.clone());
}

pub fn on_unmap_notify(context: EventContext<xcb::UnmapNotifyEvent>) {
    plugins::UnmapWindow::from_registry().do_send(context.clone());
    plugins::WindowSizer::from_registry().do_send(context.clone());
}

pub fn on_destroy_notify(context: EventContext<xcb::DestroyNotifyEvent>) {
    plugins::DestroyWindow::from_registry().do_send(context.clone());
}
