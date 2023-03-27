set -e

cargo build

unset XDG_SEAT
XEPHYR=$(command -v Xephyr)

xinit ./xinitrc -- \
    "$XEPHYR" \
        :100 \
        -ac \
        -screen 1280x800 \
        -host-cursor
