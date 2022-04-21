#!/bin/bash
SOURCE_DIR=`dirname "$0"`
TARGET_DIR=${SOURCE_DIR}/../target
ELECTRON_DIR=${SOURCE_DIR}/../src/electron

rustup target add wasm32-unknown-unknown && \
    cargo build --target wasm32-unknown-unknown --features legacy-shader && \
    cargo install wasm-bindgen-cli && \
    wasm-bindgen --out-dir ${TARGET_DIR}/generated/ --web ${TARGET_DIR}/wasm32-unknown-unknown/debug/visitlab-wgpu.wasm && \
    cd ${ELECTRON_DIR} && npm i && (npm start&) && cd ../.. && \
    python3 -m http.server
