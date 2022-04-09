#[derive(Eq, PartialEq, Hash)]
pub struct KeyPair {
    pub modifiers: u16,
    pub keysym: u32,
}

pub fn grab_key(pair: &KeyPair, conn: &xcb::Connection, root_window: xcb::Window) {
    let key_symbols = xcb_util::keysyms::KeySymbols::new(conn);
    match key_symbols.get_keycode(pair.keysym).next() {
        Some(keycode) => {
            xcb::grab_key(
                conn,
                false,
                root_window,
                pair.modifiers,
                keycode,
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            );
        }
        _ => {
            dbg!("Failed to find keycode for keysym: {}", pair.keysym);
        }
    }
}
