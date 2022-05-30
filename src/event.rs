use crate::config::Config;
use std::sync::Arc;
use actix::prelude::*;
use anyhow::Result;

#[derive(Clone, Message)]
#[rtype(result = "Result<()>")]
pub struct EventContext<E> {
    pub config: Config,
    pub conn: Arc<xcb::Connection>,
    pub event: E,
}

#[derive(Clone)]
pub struct KeyPressEvent {
    pub keycode: xcb::Keycode,
    pub mask: u16,
}

impl From<&xcb::KeyPressEvent> for KeyPressEvent {
    fn from(event: &xcb::KeyPressEvent) -> Self {
        Self {
            keycode: event.detail(),
            mask: event.state(),
        }
    }
}

#[derive(Clone)]
pub struct ConfigureRequestEvent {
    pub upper_left_x: i16,
    pub upper_left_y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub sibling: xcb::Window,
    pub window: xcb::Window,
    pub stack_mode: u8,
}

impl From<&xcb::ConfigureRequestEvent> for ConfigureRequestEvent {
    fn from(event: &xcb::ConfigureRequestEvent) -> Self {
        Self {
            upper_left_x: event.x(),
            upper_left_y: event.y(),
            width: event.width(),
            height: event.height(),
            border_width: event.border_width(),
            sibling: event.sibling(),
            window: event.window(),
            stack_mode: event.stack_mode(),
        }
    }
}

#[derive(Clone)]
pub struct MapRequestEvent {
    pub window: xcb::Window,
}

impl From<&xcb::MapRequestEvent> for MapRequestEvent {
    fn from(event: &xcb::MapRequestEvent) -> Self {
        Self {
            window: event.window(),
        }
    }
}

#[derive(Clone)]
pub struct EnterNotifyEvent {
    pub window: xcb::Window,
}

impl From<&xcb::EnterNotifyEvent> for EnterNotifyEvent {
    fn from(event: &xcb::EnterNotifyEvent) -> Self {
        Self {
            window: event.event(),
        }
    }
}

#[derive(Clone)]
pub struct UnmapNotifyEvent {
    pub window: xcb::Window,
}

impl From<&xcb::UnmapNotifyEvent> for UnmapNotifyEvent {
    fn from(event: &xcb::UnmapNotifyEvent) -> Self {
        Self {
            window: event.window(),
        }
    }
}

#[derive(Clone)]
pub struct DestroyNotifyEvent {
    pub window: xcb::Window,
}

impl From<&xcb::DestroyNotifyEvent> for DestroyNotifyEvent {
    fn from(event: &xcb::DestroyNotifyEvent) -> Self {
        Self {
            window: event.window(),
        }
    }
}
