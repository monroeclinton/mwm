mod config;
mod draw;
mod screen;
mod statusbar;

use actix::SystemService;
use anyhow::Result;
use statusbar::StatusBar;

fn main() -> Result<()> {
    let sys = actix::System::new();
    sys.block_on(async {
        // Start service.
        StatusBar::from_registry();
    });

    sys.run().expect("Failed to start statusbar.");

    Ok(())
}
