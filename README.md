# FragmentColor

FragmentColor is a cross-platform GPU programming library that provides
a simple shader composition API for:

- Javascript
- Python
- Swift
- Kotlin

```python
import fragmentcolor as fc

# Creates the shader with default values
shader = fc.Shader("my_shader.wgsl")

# It parses and binds automatically
shader.set("my-uniform", (1.0, 1.0, 1.0))

# Create renderer
renderer = fc.Renderer()

#
np_array = renderer.render(shader)
```

And thanks to [wgpu](https://github.com/gfx-rs/wgpu) as the hardware abstraction
layer, you can deploy it in a wide range of platforms:

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

## Motivation

- Reduce cognitive load on learning computer graphics techniques
- Mimic the underlying platform as close as possible without getting in the way; try to strike the right balance
- Be straightforward for the simple usecase (shadertoy-like applications)
- Do not stay in the way, allow advanced techniques and reexport some internals
- Point the user in the right direction to build inruition faster
- We're OK on sacrificing some performance for the sake of ergonomics and beginner-friendliness

## Example

> [!WARNING]  
> This library is currently under heavy development, and the API is not yet stable, meaning
> that while it is not tagged `1.0.0`, I might introduce breaking changes in minor versions.
> You can use it and test it, and I would greatly appreciate any feedback you can provide.
> If you use it in production, make sure you know what you are doing and lock the minor version.

## Building this project

### Target: web browser / wasm

Make sure you have [wasm-pack](https://rustwasm.github.io/wasm-pack/installer) installed.

```bash
cd crates/fragmentcolor-wasm
wasm-pack build --release --target web
```

The library will be available in `pkg/`.

Check the usage example in `index.html`.

### Target: desktop window

Building all:

```bash
cargo task build all
```

Building:

```bash
cd crates/fragmentcolor
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

## Supported Platforms
