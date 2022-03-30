#!/bin/bash

cargo build --target wasm32-unknown-unknown && \
wasm-bindgen --out-dir target/generated/ --web target/wasm32-unknown-unknown/debug/visitlab-wgpu.wasm && \
python3 -m http.server
