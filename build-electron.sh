#!/bin/bash
rustup target add wasm32-unknown-unknown && \
    cargo build --target wasm32-unknown-unknown --features legacy-shader && \
    cargo install wasm-bindgen-cli && \
    wasm-bindgen --out-dir target/generated/ --web target/wasm32-unknown-unknown/debug/visitlab-wgpu.wasm && \
    cd src/electron && (npm start&) && cd ../.. && \
    python3 -m http.server
