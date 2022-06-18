use crate::client::{Clients, SetActiveWindow, HandleWindowAction};
use crate::event::EventContext;
use actix::{Actor, Context, Handler, Supervised, SystemService};

#[derive(Default)]
pub struct WindowSelector;

impl Actor for WindowSelector {
    type Context = Context<Self>;
}

impl Supervised for WindowSelector {}
impl SystemService for WindowSelector {}

impl Handler<EventContext<xcb::ClientMessageEvent>> for WindowSelector {
    type Result = ();

    fn handle(&mut self, ectx: EventContext<xcb::ClientMessageEvent>, _ctx: &mut Self::Context) -> Self::Result {
        Clients::from_registry().do_send(SetActiveWindow {
            conn: ectx.conn,
            config: ectx.config,
            window: Some(ectx.event.window()),
        });
    }
}

impl Handler<EventContext<xcb::KeyPressEvent>> for WindowSelector {
    type Result = ();

    fn handle(&mut self, ectx: EventContext<xcb::KeyPressEvent>, _ctx: &mut Self::Context) -> Self::Result {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);

        for action_key_press in ectx.config.actions.iter() {
            let keycode = key_symbols
                .get_keycode(action_key_press.keysym)
                .next()
                .expect("Unknown keycode found in window_selector plugin.");

            if keycode == ectx.event.detail() && action_key_press.modifier == ectx.event.state() {
                Clients::from_registry().do_send(HandleWindowAction {
                    conn: ectx.conn.clone(),
                    config: ectx.config.clone(),
                    action: action_key_press.action.clone(),
                    window: ectx.event.event(),
                });
            }
        }
    }
}

impl Handler<EventContext<xcb::EnterNotifyEvent>> for WindowSelector {
    type Result = ();

    fn handle(&mut self, ectx: EventContext<xcb::EnterNotifyEvent>, _ctx: &mut Self::Context) -> Self::Result {
        Clients::from_registry().do_send(SetActiveWindow {
            conn: ectx.conn,
            config: ectx.config,
            window: Some(ectx.event.event()),
        });
    }
}
