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

Edit the `config.toml`:
```toml
# Border thickness
border_thickness = 2

# Space between windows
border_gap = 4

# Color of border around windows when active
active_border = 0x3b7a82

# Color of border around windows when inactive
inactive_border = 0x444444

# List of commands
[[commands]]
modifier = 0x0008 # key: l-alt
keysym = 0x0070 # key: p
command = "st"

```

# Planned features
- Multithreading (can be useful in certain cases)
- Status bar
- EWMH compliant
- Customizable layouts

# Screenshot

![Screenshot of mwm](screenshots/1.png)
