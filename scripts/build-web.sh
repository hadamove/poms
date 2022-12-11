#!/bin/bash
ROOT=`dirname "$0"`/..
cd ${ROOT}

# Install trunk if it's not already installed
cargo install --version ^0.16 trunk

# Build the web app and serve it
trunk serve --public-url "/"
