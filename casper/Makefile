prepare:
	rustup target add wasm32-unknown-unknown

build:
	@echo "Building release..."
	cargo build --release --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/bridge.wasm 2>/dev/null | true
	wasm-strip target/wasm32-unknown-unknown/release/storage.wasm 2>/dev/null | true
	wasm-strip target/wasm32-unknown-unknown/release/lock_call.wasm 2>/dev/null | true
	wasm-strip target/wasm32-unknown-unknown/release/claim_call.wasm 2>/dev/null | true

clean:
	cargo clean