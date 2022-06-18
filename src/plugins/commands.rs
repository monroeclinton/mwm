use crate::event::EventContext;
use crate::plugin::PluginHandler;
use anyhow::Result;

#[derive(Default)]
pub struct Commands;

impl PluginHandler for Commands {
    fn on_key_press(&mut self, ectx: EventContext<xcb::KeyPressEvent>) -> Result<()> {
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&ectx.conn);
        for command in &ectx.config.commands {
            if let Some(keycode) = key_symbols.get_keycode(command.keysym).next() {
                if keycode == ectx.event.detail() && command.modifier == ectx.event.state() {
                    std::process::Command::new(command.command.clone())
                        .spawn()
                        .unwrap();
                }
            }
        }

        Ok(())
    }
}
