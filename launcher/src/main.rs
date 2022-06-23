mod config;
mod draw;
mod launcher;

use launcher::Launcher;

fn main() {
    (Launcher::new()).run();
}
