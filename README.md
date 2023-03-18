# mwm

My window manager is a compositing window manager for Wayland written in Rust.

## Installation

While mwm is under heavy development, there is only a development installation guide.

### Development

#### Dependencies

- `libwayland`
- `libxkbcommon`
- `libudev`
- `libinput`
- `libgbm`
- `libseat`

#### Run

Clone this repository then:

```
cargo run
```

## Documentation

Two helpful resources for understanding Wayland are [The Wayland Protocol](https://wayland.freedesktop.org/docs/html/) and [The Wayland Book](https://wayland-book.com/). If you want to understand this project, reading those two documents is the best way to get started.

### Crates used

- [Smithay](https://smithay.github.io/smithay/smithay/index.html) provides much of what is needed to implement a Wayland compositor in Rust and re-exports useful crates like calloop, wayland_server, and wayland_protocols. It has two modules, [backend](https://smithay.github.io/smithay/smithay/backend/index.html) which has helpers to interact with the operating system/hardware, and [wayland](https://smithay.github.io/smithay/smithay/wayland/index.html) which has helpers to interact with Wayland clients. This crate has not had a release in a while, so to get new features it is built from Git instead of being pulled in from crates.io.
- [calloop](https://docs.rs/calloop/latest/calloop/) provides an event loop that can take multiple event sources, each with a callback that can respond to events.
- [wayland_server](https://docs.rs/wayland-server/latest/wayland_server/) gives access to using the Wayland protocol such as sending events to clients, receiving requests, etc.
- [wayland_protocols](https://docs.rs/wayland-protocols/latest/wayland_protocols/) gives helpers to use Wayland protocol extensions such as [XDG](https://wayland-book.com/xdg-shell-basics.html).
- [winit](https://docs.rs/winit/latest/winit/) is used as the backend to do rendering and get inputs such as key presses.
