# WebGPU in Rust

This project uses WebGPU implementation in Rust - [wgpu](https://github.com/gfx-rs/wgpu) to implement simple framework that is capable of rending protein molecules both on native and the web (WebAssembly). The project also aims to benchmark the API's maturity and performance and see if it is adequate for large molecule rendering.

## Setup

The project is currently built with Rust version `1.59.0`.
Setup instructions:

1. Install `rustup` - Rust toolchain managment: [rustup.rs - The Rust toolchain installer](https://rustup.rs/#), this should also install all necessary tools for building `rustc` - Rust compiler and `cargo` - Rust package manager.

2. The large `.pdb` files are for obvious reasons not included in the repo, you will have download them manually from the [archive](https://www.rcsb.org/structure/1AON) and put them into `src/molecules` folder.

3. Simply build and run application with `cargo run`.

## Build status

Currently, the project can be built and run on these platforms:

- MacOS Monterey

- Ubuntu 20.04

- Windows 10

## Demo

![demo](media/demo.gif)

## Todo list

- [x] basic rendering pipeline setup
- [x] parse molecule file into data for shader
- [x] render atoms using `wgsl` sphere impostor quad shader
- [ ] compile for web using `run-wasm` crate
- [ ] clean up codebase, separate components into files
- [ ] fix depth buffer quad clipping
- [ ] use depth buffer texture in the shader to visualize depth
- [ ] implement proper lighting in the shader
- [ ] implement simple GUI
- [ ] sticks and balls rendering shader
