use crate::config::get_config;
use crate::plugin::KeyPressContext;
use actix::{Actor, Context, Handler, Supervised, SystemService};
use anyhow::Result;

#[derive(Default)]
pub struct Commands;

impl Actor for Commands {
    type Context = Context<Self>;
}

impl Supervised for Commands {}
impl SystemService for Commands {}

impl Handler<KeyPressContext> for Commands {
    type Result = Result<()>;

    fn handle(&mut self, ectx: KeyPressContext, _ctx: &mut Context<Self>) -> Self::Result {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);
        let config = get_config();
        for command in config.commands {
            if let Some(keycode) = key_symbols.get_keycode(command.keysym).next() {
                if keycode == ectx.event.keycode && command.modifier == ectx.event.mask {
                    std::process::Command::new(command.command)
                        .spawn()
                        .unwrap();
                }
            }
        }

        Ok(())
    }
}
