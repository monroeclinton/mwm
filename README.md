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

# Planned features
- Plugin system
- Config file
- Multithreading (can be useful in certain cases)
- Status bar
- EWMH compliant
- Workspaces
- Customizable layouts

# Screenshot

![Screenshot of mwm](screenshots/1.png)
