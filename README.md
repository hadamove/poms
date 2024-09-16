# POMS - Portable Molecular Surface

An implementation of [Molecular Surface](https://en.wikipedia.org/wiki/Accessible_surface_area) generation and rendering, following the approach by Hermosilla et al. \[1]. It is designed to be highly portable across platforms through the use of [`wgpu`](https://github.com/gfx-rs/wgpu). In addition to molecular surface visualization, the application also offers a simpler space-filling model for molecules and basic post-processing.

\[1\]  *Hermosilla, Pedro, et al. "Interactive GPU-based generation of solvent-excluded surfaces." The Visual Computer 33.6 (2017): 869-881.*


## Requirements

- Rust toolchain (recommended installation via [rustup.rs](https://rustup.rs/#)).

> The Minimum Supported Rust Version (MSRV) is defined in `rust-toolchain.toml`. Should match the MSRV required by `wgpu`.

## 🛠️️ Building (MacOS, Windows)

```bash
cargo run
```


### Features

- `no-vsync`: uncaps the FPS, useful for performance testing:

    ```bash
    cargo run --features no-vsync
    ```

> For optimal performance, also include the `--release` flag.

## ️🌐 Building for the Web

To build for the web, we first need to add `wasm32-unknown-unknown` platform target to our Rust toolchain, which allows us to compile Rust code to WASM bytecode:

```bash
rustup target add wasm32-unknown-unknown
```

After that, you can use the following script:

```bash
./scripts/build-web.sh
```

which installs [`Trunk`](https://trunkrs.dev) and runs it with necessary configuration. This should build our WASM code and start a web server that hosts the application at `localhost:8080`.

#### What happens behind the scenes?

`Trunk` takes care of several things, which would have to be done manually otherwise:

- Builds our Rust code to WASM bytecode using `wasm32-unknown-unknown` platform target.
- Generates necessarry glue between WASM and JavaScript using [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/).
- Bundles the application into a single HTML file that can be hosted on a web server.
- Starts a web server that hosts the application.

## 🐧 Building on Linux (Ubuntu)

TODO: is this still necessary?

Due to occasional issues with Vulkan on Windows (see [1](https://github.com/rust-windowing/winit/issues/2094) and [2](https://github.com/gfx-rs/wgpu/issues/2286)), the Vulkan backend has been turned off by default. To enable it, you need to build/run the application with `--features vulkan` flag:

```bash
cargo build --features vulkan
cargo run --features vulkan
```

### Requirements

TODO: is this still necessary?

`libgtk-3-dev` and `gio-2.0` must be installed to be able to compile. To install them, run:

```bash
sudo apt install libgtk-3-dev
sudo apt install libglib2.0-dev
```
