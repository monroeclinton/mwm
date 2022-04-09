#[derive(PartialEq)]
pub struct Client {
    pub window: xcb::Window,
    pub visible: bool,
}
