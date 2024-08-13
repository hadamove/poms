# POMS - Portable Molecular Surface

This project uses [wgpu, a Rust implementation of WebGPU,](https://github.com/gfx-rs/wgpu) to implement a proof-of-concept generation and visualization of molecular surfaces (SES) that is easily portable to the majority of platforms. Besides hte SES representation, the application also supports simpler space-fill visualisation of molecules.

## Setup

### Requirements

The only prerequisite for building the application is to have Rust toolchain installed, which installs the Rust compiler `rustc` and `cargo` - Rust’s build system and package manager.

- Install rust toolchain - [rustup.rs - The Rust toolchain installer](https://rustup.rs/#).

The project is currently built with Rust version `1.65.0` (as of writing, the latest stable version). Once you have installed Rust, you can check the version by running `rustc --version` in your terminal. If you have a different version, you can install the version `1.65.0` by running `rustup default 1.65`.

- To build the application simply run in terminal:

    ```bash
    cargo build
    ```

- To run the application natively (this will also build the application if it hasn't been built yet):

    ```bash
    cargo run
    ```

- Performance is bottlenecked by the CPU in debug builds, to build/run the application in release mode, run:

    ```bash
    cargo build --release
    cargo run --release
    ```

- To uncap the FPS, add flag `--features no-vsync`:

    ```bash
    cargo run --features no-vsync
    ```

## Project Structure

- `data` - sample molecular data in PDB format, which can be used for testing
- `scripts` - scripts for building the application into WASM
- `src` - application source code in Rust
  - `src/ui` - graphical user interface code
  - `src/parser` - PDB file parser
  - `src/passes` - the majority of compute and rendering code
  - `src/utils` - common utility functions
  - `src/app.rs` - application state and logic
  - `src/context.rs` - wrapper around `wgpu`'s context - the device, surface, queue, etc.
  - `src/main.rs` - entry point of the application, handles window creation and event loop
- `target` - built application binaries and WASM files
- `Cargo.toml` - Rust package manager configuration file containing the list of dependencies
- `index.html` - HTML file that hosts the WASM application
- `LICENSE.md` - MIT license
- `README.md` - this file
- `Trunk.toml` - Trunk configuration file which handles WASM bundling

## Build Status

The project has been tested on the following platforms:

- Windows 10 (DX12) ✅
- Ubuntu 20.04 (Vulkan) ❗️ (see Building on Ubuntu)
- macOS Ventura 13.0 ✅
- Chrome Canary 110.0.5468.0 ✅

## Building for the Web

### Requirements

- To build the application for the web, we first need to add `wasm32-unknown-unknown` platform target to our Rust toolchain, which allows us to compile Rust code to WASM bytecode:

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

To target web browsers, Rust code needs to be compiled to [WebAssembly](https://webassembly.org/) (WASM), a common language supported by browsers. One of the simplest solutions is to use [Trunk](https://trunkrs.dev) - a WASM web application bundler.

To build for the web, simply run script `scripts/build-web.sh` which installs `Trunk` and runs it with necessary configuration. This will build our WASM code and start a web server that hosts the application at `http://localhost:8080`.

### What happens behind the scenes?

`Trunk` takes care of several things, which would have to be done manually otherwise:

- Builds our Rust code to WASM bytecode using `wasm32-unknown-unknown` platform target.
- Generates necessarry glue between WASM and JavaScript using [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/).
- Bundles the application into a single HTML file that can be hosted on a web server.
- Starts a web server that hosts the application.

## Building on Ubuntu

Due to occasional issues with Vulkan on Windows (see [1](https://github.com/rust-windowing/winit/issues/2094) and [2](https://github.com/gfx-rs/wgpu/issues/2286)), the Vulkan backend has been turned off by default. To enable it, you need to build/run the application with `--features vulkan` flag:

```bash
cargo build --features vulkan
cargo run --features vulkan
```

### Requirements

`libgtk-3-dev` and `gio-2.0` must be installed to be able to compile. To install them, run:

```bash
sudo apt install libgtk-3-dev
sudo apt install libglib2.0-dev
```
