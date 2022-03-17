# WebGPU in Rust

This project uses WebGPU implementation in Rust - [wgpu](https://github.com/gfx-rs/wgpu) to implement simple frameworks that is capable of rending protein molecules both on native and the web (WebAssembly). The project also aims to benchmark the API's maturity and performance and see if it is adequate for large molecule rendering.

## Setup

The project is currently built with Rust version `1.59.0`.

1. Install `rustup` (Rust toolchain managment): [rustup.rs - The Rust toolchain installer](https://rustup.rs/#), this should also install `rustc` - (Rust compiler) and `cargo` (Rust package manager).

2. Simply build and run application with `cargo run`.

## Demo

TODO: add demo gif

## Todo list

- [x] basic rendering pipeline setup
- [x] parse molecule file into data for shader
- [x] render atoms using `wgsl` sphere impostor quad shader
- [ ] compile for web using `run-wasm` crate
- [ ] clean up codebase, separate components into files
- [ ] fix depth buffer quad clipping
- [ ] implement proper lighting in the shader
- [ ] implement simple GUI
- [ ] sticks and balls rendering shader
