mod config;
mod keysym;
mod surface;
mod selector;

use selector::Selector;
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();

    let mut commands = vec![];
    while let Some(line) = stdin.lock().lines().next() {
        let command = line.expect("Unable to read stdin.");

        if command.len() > 0 {
            commands.push(command);
        }
    }

    (Selector::new(commands)).run();
}
