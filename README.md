# FragmentColor

[FragmentColor](https://fragmentcolor.org) is a cross-platform GPU programming library that is both **easy to use** and **powerful**.

The API encourages a simple shader composition workflow, where you can use **WGSL** or **GLSL** shaders as the source of truth
for visual consistency across platforms, while **avoiding the verbosity of modern GPU APIs**.

It has bindings for [**JavaScript**](./README_JS.md) (WASM), [**Python**](./README_PY.md), and draft support for **Swift** and **Kotlin**.
It targets each platform's native graphics API: **Vulkan**, **Metal**, **DirectX**, **OpenGL**, **WebGL**, and **WebGPU**.
See [Platform Support](#platform-support) for details.

Check the website for the Getting Started guide and full reference:

- **Documentation:** <https://fragmentcolor.org/welcome>
- **API Reference:** <https://fragmentcolor.org/api>

> [!NOTE]
>
> iOS and Android support is not available yet, but planned for version **v0.11.0**.
>
> See the [Roadmap](https://github.com/vista-art/fragmentcolor/blob/main/ROADMAP.md) and [Changelog](https://github.com/vista-art/fragmentcolor/blob/main/CHANGELOG.md) for details.

## Install

- Rust: add the crate to your Cargo.toml

```toml
[dependencies]
fragmentcolor = "0.10.8"
```

We also support JavaScript and Python:

- JavaScript users, see: [README_JS.md](./README_JS.md)
- Python users, see: [README_PY.md](./README_PY.md)

## Quick start

### Rust

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Shader, Pass, Frame};

// Example window. We officially support winit.
let window = fragmentcolor::headless_window(800, 600);

// Initializes a renderer and a target compatible with the OS window.
let renderer = Renderer::new();
let target = renderer.create_target(&window).await?;

// You can pass the shader as a source string, file path, or URL:
let circle = Shader::new("./path/to/circle.wgsl")?;
let triangle = Shader::new("https://fragmentcolor.org/shaders/triangle.wgsl")?;
let wgsl = r#"
struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
}

struct MyStruct {
    my_field: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> my_struct: MyStruct;

@group(0) @binding(1)
var<uniform> my_vec2: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    const vertices = array(
        vec2( -1., -1.),
        vec2(  3., -1.),
        vec2( -1.,  3.)
    );
    return VertexOutput(vec4<f32>(vertices[in_vertex_index], 0.0, 1.0));
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(my_struct.my_field, 1.0);
}
"#;

let shader = Shader::new(wgsl)?;

// The library binds and updates the uniforms automatically
shader.set("my_struct.my_field", [0.1f32, 0.8, 0.9])?;
shader.set("my_vec2", [1.0f32, 1.0])?;

// One shader is all you need to render
renderer.render(&shader, &target)?;

// But you can also combine multiple shaders in a render Pass
let pass = Pass::new("single pass");
pass.add_shader(&circle);
pass.add_shader(&triangle);
pass.add_shader(&shader);
renderer.render(&pass, &target)?;

// You can build arbitrary multi-pass graphs by declaring Pass dependencies
let blurx = Pass::new("blur x");
blurx.add_shader(&Shader::new("./shaders/blur_x.wgsl")?);
blurx.require(&pass)?; // pass renders before blurx

// Finally, you can combine multiple passes linearly in a Frame
let frame = Frame::new();
frame.add_pass(pass);
frame.add_pass(Pass::new("GUI pass"));
renderer.render(&frame, &target)?;

// To animate, simply update the uniforms in a loop
for i in 0..10 {
    circle.set("position", [i, i])?;
    renderer.render(&frame, &target)?;
}

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

### Running examples

See the examples project under `examples/rust` for implementation details.

```bash
> ./example

Available FragmentColor examples:
=================================
 1. app_healthcheck
 2. circle
 3. compute_texture
 4. fullscreen_triangle
 5. mesh_triangle
 6. mesh_two_textured_quads
 7. multiobject
 8. multipass
 9. multipass_shadows
10. particles
11. particles_1m
12. particles_compute
13. particles_splat
14. particles_splat_3d
15. push_constant_color
16. texture
17. triangle

Enter example name, number, 'q'/'quit' to quit: 
```

## Documentation & website

- Docs' source of truth lives in `docs/api`. The Rust code uses `#[lsp_doc]` to pull these docs into editor hovers.
- The website lives under `docs/website` and is automatically generated from `docs/api` at build time.
- Method pages include JavaScript and Python examples extracted from the healthcheck scripts.

For contribution guidelines and the release process, see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Platform support

Platform support is aligned with upstream [wgpu](https://github.com/gfx-rs/wgpu):

| API    | Windows      | Linux/Android   | macOS/iOS | Web (wasm)  |
| ------ | ------------ | --------------- | --------- | ----------- |
| Vulkan | ✅           | ✅              | 🌋        |             |
| Metal  |              |                 | ✅        |             |
| DX12   | ✅           |                 |           |             |
| OpenGL | 🆗 (GL 3.3+) | 🆗 (GL ES 3.0+) | 📐        | 🆗 (WebGL2) |
| WebGPU |              |                 |           | ✅          |

✅ = First Class Support  
🆗 = Downlevel/Best Effort Support  
📐 = Requires the [ANGLE](http://angleproject.org/) translation layer (GL ES 3.0 only)  
🌋 = Requires the [MoltenVK](https://vulkan.lunarg.com/sdk/home#mac) translation layer

## Limitations (planned features)

- Swift & Kotlin bindings are not supported yet, but planned for version v0.11.0.

## Common workflows

### Rust core

```bash
# Build
cargo build

# Test (all)
./test

# Lint & Format
./clippy

# Healthcheck (all platforms)
./healthcheck

# Filtered Healthcheck
./healthcheck web
./healthcheck py
```

### Web (WASM)

```bash
# Build WASM package (wasm-pack target web) and sync into local JS examples
./build_web        # add --debug for a debug build

# Run JS demos (Vite dev server) and open browser
./run_web repl     # or: ./run_web multipass | ./run_web headless

# Manual alternative
pnpm --dir examples/javascript install
pnpm --dir examples/javascript dev
```

### Python binding (local dev)

```bash
# Quick run helper: build wheel into dist/, create venv, and run an example
./run_py main      # or: ./run_py multiobject | ./run_py headless

# Manual alternative
pipx install maturin
maturin develop
pip install glfw rendercanvas
python examples/python/main.py
```

### Docs site (Astro/Starlight)

- Docs source of truth lives in docs/api and is referenced from code via `#[lsp_doc]`.
- Examples on method pages are sliced from the healthcheck scripts; no filesystem reads in docs.
- Doc examples follow async + pollster patterns.

```bash
./build_docs                    # from root, builds and tests the site
./run_docs                      # from root, runs the dev server

# or
pnpm --dir docs/website install
pnpm --dir docs/website dev      # dev server
pnpm --dir docs/website build    # static build
```
