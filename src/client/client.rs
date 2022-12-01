#[derive(Clone, Eq, PartialEq)]
pub struct Client {
    pub window: xcb::Window,
    pub workspace: Option<u8>,
    pub visible: bool,
    pub controlled: bool, // If should resize/size/configure window
    pub full_screen: bool,
    pub padding_top: u32,
}
