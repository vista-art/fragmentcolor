# FragmentColor Library

The `fragmentcolor` library aims to provide a unified API
to render **video enrichments** in the GPU (with a CPU fallback) across
multiple platforms and environments with native performance and minimal footprint.

The library is implemented in Rust,
leveraging the [wgpu](https://github.com/gfx-rs/wgpu) library for rendering,
which enables it to target a wide range of platforms:

| Backend   |  WASM   |  Linux  |  MacOS  | Windows | Android |   iOS   | CI / CD |
| :-------- | :-----: | :-----: | :-----: | :-----: | :-----: | :-----: | :-----: |
| Metal     |   no    |   no    | **Yes** |   no    |   no    | **Yes** |   no    |
| Vulkan    |   no    | **Yes** | Yes[^2] | **Yes** | **Yes** | **Yes** | **Yes** |
| OpenGL    |   no    | **Yes** | Yes[^3] | **Yes** | **Yes** | Yes[^4] |   no    |
| WebGL     | Yes[^1] |   no    |   no    |   no    |   no    |   no    |   no    |
| WebGPU    | **Yes** |   no    |   no    |   no    |   no    |   no    |   no    |
| Dx11/Dx12 |   no    |   no    |   no    | **Yes** |   no    |   no    |   no    |

[^1]: Vertex and Fragment shaders only. No support for Compute shaders.
[^2]: Available through [MoltenVK](https://github.com/KhronosGroup/MoltenVK), a translation layer to Metal.
[^3]: OpenGL is deprecated in MacOS. It runs up to version 4.1 only: no support for Compute shaders.
[^4]: OpenGL is deprecated in iOS. It runs up to version 4.1 only: no support for Compute shaders.

## Building this project

### Target: web browser / wasm

Make sure you have [wasm-pack](https://rustwasm.github.io/wasm-pack/installer) installed.

```bash
wasm-pack build --release --target web
```

The library will be available in `pkg/`.

Check the usage example in `index.html`.

### Target: desktop window

Running without building:

```bash
cargo run
```

Building:

```bash
cargo build --release
```

The dynamic library will be available in `target/release/`.

By default, the library will be built for the current platform. To build for a specific platform, use the `--target` flag:

```bash
# MacOS (Intel)
cargo build --release --target x86_64-apple-darwin

# MacOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

You can check the list of all available targets with:

```bash
rustup target list
```

Platform support is divided in Tiers, check the [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html) page for more information.
