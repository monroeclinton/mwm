pub fn grab_key(conn: &xcb_util::ewmh::Connection, modifier: u16, keysym: u32, root_window: xcb::Window) {
    let key_symbols = xcb_util::keysyms::KeySymbols::new(conn);
    match key_symbols.get_keycode(keysym).next() {
        Some(keycode) => {
            xcb::grab_key(
                conn,
                false,
                root_window,
                modifier,
                keycode,
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            );
        }
        _ => {
            dbg!("Failed to find keycode for keysym: {}", keysym);
        }
    }
}
