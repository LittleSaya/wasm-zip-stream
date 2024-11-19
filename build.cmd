@echo off

wasm-pack build --release --target web crates/wasm-zip-stream-manager

wasm-pack build --release --target web crates/wasm-zip-stream-worker
