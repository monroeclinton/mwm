mod config;
mod screen;
mod statusbar;
mod surface;

use statusbar::StatusBar;

#[tokio::main]
async fn main() {
    (StatusBar::new()).run().await;
}
