mod client;
mod config;
mod event;
mod listeners;
mod macros;
mod plugins;
mod window_manager;

use actix::SystemService;
use window_manager::WindowManager;
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let sys = actix::System::new();
    sys.block_on(async {
        // Start service.
        WindowManager::from_registry();
    });

    sys.run().context("Failed to start mwm.")?;

    Ok(())
}
