raspberry-local:
	cargo build --target="armv7-unknown-linux-gnueabihf" && \
	pscp -pw 'raspberry' -r ./target/armv7-unknown-linux-gnueabihf/debug/relay pi@raspberrypi.local:/home/pi/bin && \
  plink -pw 'raspberry' pi@raspberrypi.local './bin/relay'
