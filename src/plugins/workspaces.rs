use crate::client::{Clients, VisibleWindows};
use crate::event::{EventContext, KeyPressEvent, MapRequestEvent};
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

impl Handler<EventContext<KeyPressEvent>> for Workspaces {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<KeyPressEvent>, _ctx: &mut Self::Context) -> Self::Result {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);

        let mut active_workspace = self.active_workspace;
        for workspace in 1..9 {
            let keycode = key_symbols
                .get_keycode(x11::keysym::XK_0 + workspace as u32)
                .next()
                .expect("Unknown keycode found in workspaces plugin.");

            if keycode == ectx.event.keycode && ectx.config.workspace_modifier == ectx.event.mask {
                active_workspace = workspace;
            }
        }

        if self.active_workspace == active_workspace {
            return Box::pin(actix::fut::wrap_future::<_, Self>(async {
                Ok(())
            }));
        }

        drop(key_symbols);

        Self::from_registry()
            .send(SetActiveWorkspace {
                conn: ectx.conn,
                workspace: active_workspace,
            })
            .into_actor(self)
            .map(|_, _, _| Ok(()))
            .boxed_local()
    }
}

impl Handler<EventContext<MapRequestEvent>> for Workspaces {
    type Result = Result<()>;

    fn handle(&mut self, ectx: EventContext<MapRequestEvent>, _ctx: &mut Context<Self>) -> Self::Result {
        let active_windows = self.workspaces
            .get_mut(&self.active_workspace)
            .expect("Unable to find workspace.");

        active_windows.push(ectx.event.window);

        Ok(())
    }
}

struct SetActiveWorkspace {
    conn: Arc<xcb::Connection>,
    workspace: u8,
}

impl Message for SetActiveWorkspace {
    type Result = Result<()>;
}

impl Handler<SetActiveWorkspace> for Workspaces {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: SetActiveWorkspace, _ctx: &mut Self::Context) -> Self::Result {
        self.active_workspace = msg.workspace;

        let active_windows = self.workspaces
            .get_mut(&self.active_workspace)
            .expect("Unable to find workspace.");

        Clients::from_registry()
            .send(VisibleWindows {
                conn: msg.conn,
                windows: active_windows.clone(),
            })
            .into_actor(self)
            .map(|_, _, _| Ok(()))
            .boxed_local()
    }
}
