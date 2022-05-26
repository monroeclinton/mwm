use crate::event;
use std::sync::Arc;
use actix::Message;
use anyhow::Result;

#[derive(Clone)]
pub struct EventContext<E> {
    pub conn: Arc<xcb::Connection>,
    pub event: E,
}

pub type KeyPressContext = EventContext<event::KeyPressEvent>;

impl Message for KeyPressContext {
    type Result = Result<()>;
}

pub type ConfigureRequestContext = EventContext<event::ConfigureRequestEvent>;

impl Message for ConfigureRequestContext {
    type Result = Result<()>;
}

pub type MapRequestContext = EventContext<event::MapRequestEvent>;

impl Message for MapRequestContext {
    type Result = Result<()>;
}

pub type EnterNotifyContext = EventContext<event::EnterNotifyEvent>;

impl Message for EnterNotifyContext {
    type Result = Result<()>;
}

pub type UnmapNotifyContext = EventContext<event::UnmapNotifyEvent>;

impl Message for UnmapNotifyContext {
    type Result = Result<()>;
}
