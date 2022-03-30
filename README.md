# WebGPU in Rust

This project uses WebGPU implementation in Rust - [wgpu](https://github.com/gfx-rs/wgpu) to implement a simple framework that is capable of rending protein molecules both on native and the web (WebAssembly). The project also aims to benchmark the API's maturity and performance and see if it is adequate for large molecule rendering.

## Setup

The project is currently built with Rust version `1.59.0`.
Setup instructions:

1. Install `rustup` - Rust toolchain management: [rustup.rs - The Rust toolchain installer](https://rustup.rs/#), this should also install all necessary tools for building `rustc` - Rust compiler and `cargo` - Rust package manager.

2. The large `.pdb` files are, for obvious reasons, not included in the repo; you will have to download them manually from the [archive](https://www.rcsb.org/structure/1AON) and put them into `src/molecules` folder.

3. Build and run the application with `cargo run`.

## Build status

Currently, the project can be built and run on these platforms:

- macOS Monterey

- Ubuntu 20.04

- Windows 10

- Chrome Canary 102.0.4972

Note:
Running in Firefox Nightly fails due to Firefox's outdated shader syntax (Chrome Canary has recently abandoned the old wgsl syntax completely).

## Demo

![demo](media/demo.gif)

## Building for the web

To target web browsers, Rust code needs to be compiled to [WebAssembly](https://webassembly.org/), a common language supported by browsers. For this purpose, we use `wasm32-unknown-unknown` as target and use crate `wasm-bindgen` that generates the needed JavaScript glue. Finally, we set up a simple web server that will host our application, which uses `index.html` as an entry point to our WebAssembly bytecode. To put in simple steps:

1. Compile to wasm: `cargo build --target wasm32-unknown-unknown`

2. Add JS glue: `wasm-bindgen --out-dir target/generated/ --web target/wasm32-unknown-unknown/debug/visitlab-wgpu.wasm`

3. Host the application: `python3 -m http.server` (you can use any other http server implementation, although it is important to run the server on port `8000`, see `parser.rs` for detailed information).

4. The application should be accessible at `http://localhost:8000/` in a browser with WebGPU support (e.g., Chrome Canary).

![Chrome Canary](media/chrome-canary.png)

Alternatively, you can use `build-web.sh` script to run these three commands.

## Todo list

- [x] basic rendering pipeline setup
- [x] parse molecule file into data for shader
- [x] render atoms using `wgsl` sphere impostor quad shader
- [x] compilation for web browsers
- [ ] implement proper lighting in the shader
- [ ] fix depth buffer quad clipping
- [ ] use depth buffer texture in the shader to visualize depth
- [ ] implement simple GUI
- [ ] sticks and balls rendering shader
