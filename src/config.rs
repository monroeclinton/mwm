use crate::key::KeyPair;
use crate::plugin::PluginHandler;
use std::collections::HashMap;

pub type Command = Box<dyn Fn() -> std::process::Command>;

pub struct Config {
    pub commands: HashMap<KeyPair, Command>,
    pub plugins: Vec<Box<dyn PluginHandler>>,
}
