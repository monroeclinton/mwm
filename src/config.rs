use crate::key::{KeyPair};
use crate::window_manager::{Handler};
use std::collections::{HashMap};

pub struct Config {
    pub keys: HashMap<KeyPair, Handler>,
}
