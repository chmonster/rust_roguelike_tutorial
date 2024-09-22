#!/bin/sh
echo "cargo build"
cargo build --release --target wasm32-unknown-unknown
echo "wasm-bindgen"
wasm-bindgen ./target/wasm32-unknown-unknown/release/rust_roguelike.wasm --out-dir wasm --no-modules --no-typescript
