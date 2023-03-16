all: build

build:
	cargo build --release -p mwm

clean:
	cargo clean

install: all
	sudo cp target/release/mwm /usr/local/bin/
