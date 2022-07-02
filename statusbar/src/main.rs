mod config;
mod surface;
mod screen;
mod statusbar;

use statusbar::StatusBar;

#[tokio::main]
async fn main() {
    (StatusBar::new()).run().await;
}
