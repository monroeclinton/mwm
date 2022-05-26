use crate::event::{EventContext, ConfigureRequestEvent};
use actix::{Actor, Context, Handler, Supervised, SystemService};
use anyhow::Result;

#[derive(Default)]
pub struct ConfigureWindow;

impl Actor for ConfigureWindow {
    type Context = Context<Self>;
}

impl Supervised for ConfigureWindow {}
impl SystemService for ConfigureWindow {}

impl Handler<EventContext<ConfigureRequestEvent>> for ConfigureWindow {
    type Result = Result<()>;

    fn handle(&mut self, ectx: EventContext<ConfigureRequestEvent>, _ctx: &mut Context<Self>) -> Self::Result {
        let values = vec![
            (xcb::CONFIG_WINDOW_X as u16, ectx.event.upper_left_x as u32),
            (xcb::CONFIG_WINDOW_Y as u16, ectx.event.upper_left_y as u32),
            (xcb::CONFIG_WINDOW_WIDTH as u16, ectx.event.width as u32),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, ectx.event.height as u32),
            (
                xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
                ectx.event.border_width as u32,
            ),
            (xcb::CONFIG_WINDOW_SIBLING as u16, ectx.event.sibling as u32), // Default: NONE
            (
                xcb::CONFIG_WINDOW_STACK_MODE as u16,
                ectx.event.stack_mode as u32,
            ), // Default: STACK_MODE_ABOVE
        ];

        xcb::configure_window(&ectx.conn, ectx.event.window, &values);

        Ok(())
    }
}
