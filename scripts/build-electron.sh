#!/bin/bash
ROOT=`dirname "$0"`/..
ELECTRON_DIR="./src-electron"
cd ${ROOT}

# Install trunk if it's not already installed
cargo install --version ^0.16 trunk

# Build the WASM app 
trunk build

# Install and build the electron app
npm install --prefix ${ELECTRON_DIR}
npm run start --prefix ${ELECTRON_DIR}
