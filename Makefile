all: build

build:
	cargo build --release -p mwm -p selector -p statusbar

clean:
	cargo clean

install: all
	sudo cp target/release/mwm /usr/local/bin/
	sudo cp target/release/selector /usr/local/bin/
	sudo cp target/release/statusbar /usr/local/bin/
