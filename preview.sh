set -e

cargo build

unset XDG_SEAT
XEPHYR=$(command -v Xephyr)

xinit ./xinitrc -- \
    "$XEPHYR" \
        :100 \
        -ac \
        -screen 1200x800 \
        -host-cursor
