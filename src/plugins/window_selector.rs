use crate::config::Config;
use crate::client::{Clients, SetActiveWindow, HandleWindowAction};
use crate::event::EventContext;
use std::sync::Arc;
use actix::{Actor, ActorFutureExt, Context, Handler, ResponseActFuture, Supervised, SystemService, WrapFuture};
use anyhow::Result;

#[derive(Default)]
pub struct WindowSelector;

impl WindowSelector {
    fn set_active_window(
        &self,
        conn: Arc<xcb_util::ewmh::Connection>,
        config: Arc<Config>,
        window: Option<xcb::Window>
    ) -> ResponseActFuture<Self, Result<()>>{
        Clients::from_registry()
            .send(SetActiveWindow {
                conn,
                config,
                window,
            })
            .into_actor(self)
            .map(|_, _, _| { Ok(()) })
            .boxed_local()
    }
}

impl Actor for WindowSelector {
    type Context = Context<Self>;
}

impl Supervised for WindowSelector {}
impl SystemService for WindowSelector {}

impl Handler<EventContext<xcb::ClientMessageEvent>> for WindowSelector {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::ClientMessageEvent>, _ctx: &mut Self::Context) -> Self::Result {
        self.set_active_window(ectx.conn, ectx.config, Some(ectx.event.window()))
    }
}

impl Handler<EventContext<xcb::KeyPressEvent>> for WindowSelector {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::KeyPressEvent>, _ctx: &mut Self::Context) -> Self::Result {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);

        let mut action = None;
        for action_key_press in ectx.config.actions.iter() {
            let keycode = key_symbols
                .get_keycode(action_key_press.keysym)
                .next()
                .expect("Unknown keycode found in window_selector plugin.");

            if keycode == ectx.event.detail() && action_key_press.modifier == ectx.event.state() {
                action = Some(action_key_press.action.clone());
                break;
            }
        }

        drop(key_symbols);

        if let Some(action) = action {
            Clients::from_registry()
                .send(HandleWindowAction {
                    conn: ectx.conn,
                    config: ectx.config,
                    action,
                    window: ectx.event.event(),
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

impl Handler<EventContext<xcb::EnterNotifyEvent>> for WindowSelector {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, ectx: EventContext<xcb::EnterNotifyEvent>, _ctx: &mut Self::Context) -> Self::Result {
        self.set_active_window(ectx.conn, ectx.config, Some(ectx.event.event()))
    }
}
