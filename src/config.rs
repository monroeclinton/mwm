use crate::key::KeyPair;
use crate::plugin::PluginHandler;
use std::collections::HashMap;

pub type Command = Box<dyn Fn() -> std::process::Command>;

pub struct Config {
    pub commands: HashMap<KeyPair, Command>,
    pub plugins: Vec<Box<dyn PluginHandler>>,
    pub border_thickness: u32,
    pub border_gap: u32,
    pub active_border: u32,
    pub inactive_border: u32,
}
