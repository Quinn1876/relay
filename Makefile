raspberry-local:
	cargo build --target="armv7-unknown-linux-gnueabihf" --features "socketcan"

on-pi:
	cargo build --features "socketcan"
