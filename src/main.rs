mod client;
mod config;
mod event;
mod handler;
mod key;
mod plugin;
mod plugins;
mod screen;
mod window_manager;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use window_manager::WindowManager;

#[tokio::main]
async fn main() {
    // Setup tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // Run window manager
    WindowManager::new().run();
}
