use crate::client::Clients;
use crate::config::Config;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

pub struct EventContext<E> {
    pub clients: Arc<Mutex<Clients>>,
    pub config: Arc<Config>,
    pub conn: Arc<xcb_util::ewmh::Connection>,
    pub event: E,
}

impl Clone for EventContext<xcb::ClientMessageEvent> {
    fn clone(&self) -> Self {
        let data = match self.event.format() {
            16 => {
                let slice = <[u16; 10]>::try_from(self.event.data().data16()).unwrap_or([0; 10]);
                xcb::ClientMessageData::from_data16(slice)
            }
            32 => {
                let slice = <[u32; 5]>::try_from(self.event.data().data32()).unwrap_or([0; 5]);
                xcb::ClientMessageData::from_data32(slice)
            }
            _ => {
                let slice = <[u8; 20]>::try_from(self.event.data().data8()).unwrap_or([0; 20]);
                xcb::ClientMessageData::from_data8(slice)
            }
        };

        let event = xcb::ClientMessageEvent::new(
            self.event.format(),
            self.event.window(),
            self.event.type_(),
            data,
        );

        Self {
            clients: self.clients.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            event,
        }
    }
}

impl Clone for EventContext<xcb::KeyPressEvent> {
    fn clone(&self) -> Self {
        let event = xcb::KeyPressEvent::new(
            self.event.response_type(),
            self.event.detail(),
            self.event.time(),
            self.event.root(),
            self.event.event(),
            self.event.child(),
            self.event.root_x(),
            self.event.root_y(),
            self.event.event_x(),
            self.event.event_y(),
            self.event.state(),
            self.event.same_screen(),
        );

        Self {
            clients: self.clients.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            event,
        }
    }
}

impl Clone for EventContext<xcb::ConfigureRequestEvent> {
    fn clone(&self) -> Self {
        let event = xcb::ConfigureRequestEvent::new(
            self.event.stack_mode(),
            self.event.parent(),
            self.event.window(),
            self.event.sibling(),
            self.event.x(),
            self.event.y(),
            self.event.width(),
            self.event.height(),
            self.event.border_width(),
            self.event.value_mask(),
        );

        Self {
            clients: self.clients.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            event,
        }
    }
}

impl Clone for EventContext<xcb::MapRequestEvent> {
    fn clone(&self) -> Self {
        let event = xcb::MapRequestEvent::new(self.event.parent(), self.event.window());

        Self {
            clients: self.clients.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            event,
        }
    }
}

impl Clone for EventContext<xcb::PropertyNotifyEvent> {
    fn clone(&self) -> Self {
        let event = xcb::PropertyNotifyEvent::new(
            self.event.window(),
            self.event.atom(),
            self.event.time(),
            self.event.state(),
        );

        Self {
            clients: self.clients.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            event,
        }
    }
}

impl Clone for EventContext<xcb::EnterNotifyEvent> {
    fn clone(&self) -> Self {
        let event = xcb::EnterNotifyEvent::new(
            self.event.response_type(),
            self.event.detail(),
            self.event.time(),
            self.event.root(),
            self.event.event(),
            self.event.child(),
            self.event.root_x(),
            self.event.root_y(),
            self.event.event_x(),
            self.event.event_y(),
            self.event.state(),
            self.event.mode(),
            self.event.same_screen_focus(),
        );

        Self {
            clients: self.clients.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            event,
        }
    }
}

impl Clone for EventContext<xcb::UnmapNotifyEvent> {
    fn clone(&self) -> Self {
        let event = xcb::UnmapNotifyEvent::new(
            self.event.event(),
            self.event.window(),
            self.event.from_configure(),
        );

        Self {
            clients: self.clients.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            event,
        }
    }
}

impl Clone for EventContext<xcb::DestroyNotifyEvent> {
    fn clone(&self) -> Self {
        let event = xcb::DestroyNotifyEvent::new(self.event.event(), self.event.window());

        Self {
            clients: self.clients.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            event,
        }
    }
}
