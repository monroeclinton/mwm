# mwm 
My window manager that is a work in progress. Currently hacky

# Installation
Clone this repo then:
```
cargo build --release
```
Put the binary in your path, then add this to your `xinitrc`:
```
exec mwm
```

# Configuring

In the `main.rs` file edit the config struct.
```rust
let config = crate::config::Config {
    // Map commands to keypress
    commands: key_map!(
        (
            KeyPair { // If press alt-p then open st (suckless.org simple terminal)
                modifiers: xcb::MOD_MASK_1 as u16,
                keysym: x11::keysym::XK_p,
            },
            Handler {
                command: Some(Box::new(|| Command::new("st"))),
                event: None,
            }
        )
    ),
    // Load plugins
    plugins: vec![
        Box::new(plugins::load_window_mapper_plugin()),
    ],
    // Border config
    border_thickness: 2,
    border_gap: 4,
    active_border: 0x3b7a82,
    inactive_border: 0x444444,
};
```

# Planned features
- Multithreading (can be useful in certain cases)
- Status bar
- EWMH compliant
- Customizable layouts

# Screenshot

![Screenshot of mwm](screenshots/1.png)
