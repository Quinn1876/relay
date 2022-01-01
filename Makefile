raspberry-local:
	cargo build --target="armv7-unknown-linux-gnueabihf" --features "socketcan"

on-pi:
	cargo build --features "socketcan"

generate-bindings-canota:
	bindgen ./canbus-canota/canota.h -o canota-sys/src/bindings.rs --whitelist-function '^canota_.*'
