use crate::plugins;
use crate::event::{
    EventContext, KeyPressEvent, ConfigureRequestEvent, MapRequestEvent,
    EnterNotifyEvent, UnmapNotifyEvent
};
use crate::macros::ignore_results;
use actix::SystemService;

pub async fn on_key_press(context: EventContext<KeyPressEvent>) {
    ignore_results!(plugins::Commands::from_registry().send(context.clone()).await);
    ignore_results!(plugins::WindowSelector::from_registry().send(context.clone()).await);
    ignore_results!(plugins::Workspaces::from_registry().send(context.clone()).await);
}

pub async fn on_configure_request(context: EventContext<ConfigureRequestEvent>) {
    ignore_results!(plugins::ConfigureWindow::from_registry().send(context).await);
}

pub async fn on_map_request(context: EventContext<MapRequestEvent>) {
    ignore_results!(plugins::MapWindow::from_registry().send(context.clone()).await);
    ignore_results!(plugins::WindowSizer::from_registry().send(context.clone()).await);
    ignore_results!(plugins::Workspaces::from_registry().send(context.clone()).await);
}

pub async fn on_enter_notify(context: EventContext<EnterNotifyEvent>) {
    ignore_results!(plugins::WindowSelector::from_registry().send(context.clone()).await);
}

pub async fn on_unmap_notify(context: EventContext<UnmapNotifyEvent>) {
    ignore_results!(plugins::UnmapWindow::from_registry().send(context.clone()).await);
}
