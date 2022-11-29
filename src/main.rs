mod client;
mod config;
mod event;
mod handler;
mod key;
mod plugin;
mod plugins;
mod screen;
mod window_manager;

use window_manager::WindowManager;

#[tokio::main]
async fn main() {
    WindowManager::new().run();
}
