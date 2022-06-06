use crate::client::{Clients, VisibleWindows};
use crate::event::{EventContext, DestroyNotifyEvent, KeyPressEvent, MapRequestEvent};
use std::collections::HashMap;
use std::sync::Arc;
use actix::{
    Actor, ActorFutureExt, Context, Handler, Message, ResponseActFuture,
    Supervised, SystemService, WrapFuture
};
use anyhow::Result;

pub struct Workspaces {
    active_workspace: u8,
    workspaces: HashMap<u8, Vec<xcb::Window>>,
}

impl Default for Workspaces {
    fn default() -> Self {
        let mut workspaces = HashMap::new();

        for w in 1..9 {
            workspaces.insert(w, Vec::new());
        }

        Self {
            active_workspace: 1,
            workspaces,
        }
    }
}

impl Actor for Workspaces {
    type Context = Context<Self>;
}

impl Supervised for Workspaces {}
impl SystemService for Workspaces {}

impl Handler<EventContext<xcb::ClientMessageEvent>> for Workspaces {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::ClientMessageEvent>, _ctx: &mut Self::Context) -> Self::Result {
        Clients::from_registry()
            .send(SetActiveWorkspace {
                conn: ectx.conn,
                workspace: 1,
            })
            .into_actor(self)
            .map(|_, _, _| Ok(()))
            .boxed_local()
    }
}

impl Handler<EventContext<xcb::KeyPressEvent>> for Workspaces {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::KeyPressEvent>, _ctx: &mut Self::Context) -> Self::Result {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);

        let mut active_workspace: Option<u8> = None;
        for workspace in 1..9 {
            let keycode = key_symbols
                .get_keycode(x11::keysym::XK_0 + workspace as u32)
                .next()
                .expect("Unknown keycode found in workspaces plugin.");

            if keycode == ectx.event.detail() && ectx.config.workspace_modifier == ectx.event.state() {
                active_workspace = Some(workspace);
            }
        }

        drop(key_symbols);

        if let Some(workspace) = active_workspace {
            Clients::from_registry()
                .send(SetActiveWorkspace {
                    conn: ectx.conn,
                    workspace,
                })
                .into_actor(self)
                .map(|_, _, _| Ok(()))
                .boxed_local()
        } else {
            Box::pin(actix::fut::wrap_future::<_, Self>(async {
                Ok(())
            }))
        }
    }
}
