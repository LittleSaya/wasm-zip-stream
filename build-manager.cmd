@echo off

cd %~dp0

wasm-pack build --release --target web crates/wasm-zip-stream-manager
