#!/bin/bash
rustup target add wasm32-unknown-unknown && \
    cargo build --target wasm32-unknown-unknown && \
    cargo install wasm-bindgen-cli && \
    wasm-bindgen --out-dir target/generated/ --web target/wasm32-unknown-unknown/debug/visitlab-wgpu.wasm && \
    python3 -m http.server
