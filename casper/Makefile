prepare:
	rustup target add wasm32-unknown-unknown

build:
	cd bridge && cargo build --release --target wasm32-unknown-unknown
	wasm-strip bridge/target/wasm32-unknown-unknown/release/bridge.wasm 2>/dev/null | true

clean:
	cd bridge && cargo clean