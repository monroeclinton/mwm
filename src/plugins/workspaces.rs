use crate::client::{Clients, SetActiveWorkspace, SetActiveWindow};
use crate::event::EventContext;
use actix::{Actor, ActorFutureExt, Context, Handler, ResponseActFuture, Supervised, SystemService, WrapFuture};
use anyhow::Result;

#[derive(Default)]
pub struct Workspaces;

impl Actor for Workspaces {
    type Context = Context<Self>;
}

impl Supervised for Workspaces {}
impl SystemService for Workspaces {}

impl Handler<EventContext<xcb::ClientMessageEvent>> for Workspaces {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::ClientMessageEvent>, _ctx: &mut Self::Context) -> Self::Result {
        if ectx.event.type_() == ectx.conn.CURRENT_DESKTOP() {
            Clients::from_registry()
                .send(SetActiveWorkspace {
                    conn: ectx.conn,
                    workspace: 1,
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

impl Handler<EventContext<xcb::KeyPressEvent>> for Workspaces {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::KeyPressEvent>, _ctx: &mut Self::Context) -> Self::Result {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);

        let mut active_workspace: Option<u8> = None;
        for workspace in 1..=9 {
            let keycode = key_symbols
                .get_keycode(x11::keysym::XK_0 + workspace as u32)
                .next()
                .expect("Unknown keycode found in workspaces plugin.");

            if keycode == ectx.event.detail() && ectx.config.workspace_modifier == ectx.event.state() {
                active_workspace = Some(workspace);
                break;
            }
        }

        drop(key_symbols);

        if let Some(workspace) = active_workspace {
            let conn = ectx.conn.clone();

            Clients::from_registry()
                .send(SetActiveWorkspace {
                    conn,
                    workspace,
                })
                .into_actor(self)
                .then(|_, actor, _ctx| {
                    Clients::from_registry()
                        .send(SetActiveWindow {
                            conn: ectx.conn,
                            config: ectx.config,
                            window: None,
                        })
                        .into_actor(actor)
                })
                .map(|_, _, _| Ok(()))
                .boxed_local()
        } else {
            Box::pin(actix::fut::wrap_future::<_, Self>(async {
                Ok(())
            }))
        }
    }
}
