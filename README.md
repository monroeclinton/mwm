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

There are config toml files in the `src` and `statusbar` directories
that you can customize.

You can make a symlink to these files in `~/.config/mwm/`.

# Screenshots

![Screenshot of mwm](screenshots/3.png)
![Screenshot of mwm](screenshots/4.png)
