use crate::plugins;
use crate::event::EventContext;
use crate::macros::ignore_results;
use actix::SystemService;

pub async fn on_client_message(context: EventContext<xcb::ClientMessageEvent>) {
    ignore_results!(plugins::WindowSelector::from_registry().send(context.clone()).await);
    ignore_results!(plugins::Workspaces::from_registry().send(context.clone()).await);
}

pub async fn on_key_press(context: EventContext<xcb::KeyPressEvent>) {
    ignore_results!(plugins::Commands::from_registry().send(context.clone()).await);
    ignore_results!(plugins::WindowSelector::from_registry().send(context.clone()).await);
    ignore_results!(plugins::Workspaces::from_registry().send(context.clone()).await);
}

pub async fn on_configure_request(context: EventContext<xcb::ConfigureRequestEvent>) {
    ignore_results!(plugins::ConfigureWindow::from_registry().send(context).await);
}

pub async fn on_map_request(context: EventContext<xcb::MapRequestEvent>) {
    ignore_results!(plugins::MapWindow::from_registry().send(context.clone()).await);
    ignore_results!(plugins::WindowSizer::from_registry().send(context.clone()).await);
}

pub async fn on_property_notify(context: EventContext<xcb::PropertyNotifyEvent>) {
    ignore_results!(plugins::WindowSizer::from_registry().send(context.clone()).await);
    ignore_results!(plugins::ConfigureWindow::from_registry().send(context).await);
}

pub async fn on_enter_notify(context: EventContext<xcb::EnterNotifyEvent>) {
    ignore_results!(plugins::WindowSelector::from_registry().send(context.clone()).await);
}

pub async fn on_unmap_notify(context: EventContext<xcb::UnmapNotifyEvent>) {
    ignore_results!(plugins::UnmapWindow::from_registry().send(context.clone()).await);
    ignore_results!(plugins::WindowSizer::from_registry().send(context.clone()).await);
}

pub async fn on_destroy_notify(context: EventContext<xcb::DestroyNotifyEvent>) {
    ignore_results!(plugins::DestroyWindow::from_registry().send(context.clone()).await);
}
