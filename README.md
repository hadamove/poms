# Interactive SES generation in WebGPU

This project uses WebGPU implementation in Rust - [wgpu](https://github.com/gfx-rs/wgpu) to implement a proof-of-concept generation and visualization of molecular surfaces (SES) on multiple platforms. The application also supports simpler space-fill visualisation of molecules.

## Setup

The project is currently built with Rust version `1.63.0`.
Setup instructions:

1. Install `rustup` - Rust toolchain management: [rustup.rs - The Rust toolchain installer](https://rustup.rs/#), this should also install all necessary tools for building `rustc` - Rust compiler and `cargo` - Rust package manager.

2. Build and run the application with `cargo run` in root directory.

## Build status

Currently, the project can be built on multiple platform, tested platforms include:

- macOS Ventura 13.0

- Ubuntu 20.04

- Windows 10

- Chrome Canary 109.0.5403.0

- (Electron 22.0.0-nightly.20220926)

## Building for the web

To target web browsers, Rust code needs to be compiled to [WebAssembly](https://webassembly.org/), a common language supported by browsers. For this purpose, we use `wasm32-unknown-unknown` as target and use crate `wasm-bindgen` that generates the needed JavaScript glue. Finally, we set up a simple web server that will host our application, which uses `index.html` as an entry point to our WebAssembly bytecode. To put in simple steps:

> Alternatively, you can use `scripts/build-web.sh` script to run the following five commands for you.

1. Add wasm compilation target: `rustup target add wasm32-unknown-unknown`

2. Compile to wasm: `cargo build --target wasm32-unknown-unknown`

3. Install `wasm-bindgen` for generating JS glue: `cargo install wasm-bindgen-cli`

4. Generate JS glue: `wasm-bindgen --out-dir target/generated/ --web target/wasm32-unknown-unknown/debug/visitlab-wgpu.wasm`

5. Host the application: `python3 -m http.server`

6. The application should be accessible at `http://localhost:8000/` in a browser with WebGPU support (e.g., Chrome Canary).

![Chrome Canary](media/chrome-canary.png)

## Building Electron Application

Building the Electron Application requires `Node.js`, which can be downloded from <https://nodejs.org/en/download/>.

> Alternatively, you can use `scripts/build-electron.sh` script to run the following commands for you.

1. Build and host the web application either manually as described above in **Building for the web** section or using `scripts/build-web.sh` script.
2. In a new terminal window navigate to `src/electron` and run `npm i` to install Electron dependencies.
3. Run `npm start` to run the Electron application.
