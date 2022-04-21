#!/bin/bash
SOURCE_DIR=`dirname "$0"`
TARGET_DIR=${SOURCE_DIR}/../target

rustup target add wasm32-unknown-unknown && \
    cargo build --target wasm32-unknown-unknown && \
    cargo install wasm-bindgen-cli && \
    wasm-bindgen --out-dir ${TARGET_DIR}/generated/ --web ${TARGET_DIR}/wasm32-unknown-unknown/debug/visitlab-wgpu.wasm && \
    python3 -m http.server -d ${SOURCE_DIR}/..
