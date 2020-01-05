static/desune-counter-frontend.wasm: target/wasm32-unknown-unknown/release/desune-counter-frontend.wasm
	cp target/wasm32-unknown-unknown/release/desune-counter-frontend.wasm static/desune-counter-frontend.wasm

target/wasm32-unknown-unknown/release/desune-counter-frontend.wasm: desune-counter-frontend/src/main.rs
	cd desune-counter-frontend && cargo web build --release
