# FragmentColor

[FragmentColor](https://fragmentcolor.org) is a cross-platform GPU programming library implemented in Rust and [wgpu](https://wgpu.rs).

It has bindings for [**JavaScript**](./README_JS.md) (WASM), [**Python**](./README_PY.md), and draft support for **Swift** and **Kotlin**.
It targets each platform's native graphics API: **Vulkan**, **Metal**, **DirectX**, **OpenGL**, **WebGL**, and **WebGPU**.
See [Platform Support](#platform-support) for details.

The API encourages a simple shader composition workflow. You can use **WGSL** or **GLSL** shaders
for visual consistency across platforms, while avoiding the verbosity of modern GPU APIs.

Check the website for the Getting Started guide and full reference:

- **Documentation:** <https://fragmentcolor.org/welcome>
- **API Reference:** <https://fragmentcolor.org/api>

> ⚠️ Status
>
> FragmentColor is still maturing. Expect minor versions to introduce changes as features evolve.
> See the [Roadmap](https://github.com/vista-art/fragmentcolor/blob/main/ROADMAP.md) and [Changelog](https://github.com/vista-art/fragmentcolor/blob/main/CHANGELOG.md) for details.

## Install

- Rust: add the crate to your Cargo.toml

```toml
[dependencies]
fragmentcolor = "0.10.7"
```

We also support JavaScript and Python:
- JavaScript users, see: [README_JS.md](./README_JS.md)
- Python users, see: [README_PY.md](./README_PY.md)

## Quick start

### Rust

```rust
use fragmentcolor::{Renderer, Shader, Pass, Frame};
use pollster::block_on;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    block_on(async {
        // Initializes a renderer and a target (offscreen example here)
        let renderer = Renderer::new();
        let target = renderer.create_texture_target([800, 600]).await?;

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
        let mut shader = Shader::new(wgsl)?;

        // The library binds and updates the uniforms automatically
        shader.set("my_struct.my_field", [0.1f32, 0.8, 0.9])?;
        shader.set("my_vec2", [1.0f32, 1.0])?;

        // One shader is all you need to render
        renderer.render(&shader, &target)?;

        // But you can also combine multiple shaders in a render Pass
        let mut rpass = Pass::new("single pass");
        rpass.add_shader(&circle);
        rpass.add_shader(&triangle);
        rpass.add_shader(&shader);
        renderer.render(&rpass, &target)?;

        // Finally, you can combine multiple passes in a Frame
        let mut frame = Frame::new();
        frame.add_pass(rpass);
        frame.add_pass(Pass::new("GUI pass"));
        renderer.render(&frame, &target)?;

        // To animate, simply update the uniforms in a loop
        // (Pseudo-code)
        // loop {
        //     circle.set("position", [0.0f32, 0.0])?;
        //     renderer.render(&frame, &target)?;
        // }

        Ok(())
    })
}
```

### Rust (Desktop)

See the examples project under `examples/rust`:

```bash
cargo run -p fce --example circle
cargo run -p fce --example triangle
cargo run -p fce --example multiobject
cargo run -p fce --example multipass
```

## Documentation & website

- Docs source of truth lives in `docs/api`. The Rust code uses `#[lsp_doc]` to pull these docs into editor hovers.
- The website lives under `docs/website` and is automatically generated from `docs/api` at build time.
- Method pages include JavaScript and Python examples extracted from the healthcheck scripts.

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

- Swift & Kotlin bindings are not supported yet, but planned for version v0.10.8.

## Common workflows

### Rust core

```bash
# Build
cargo build

# Test (all)
cargo test

# Lint (deny warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt
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
pnpm --dir docs/website install
pnpm --dir docs/website dev      # dev server
pnpm --dir docs/website build    # static build
```

## Development

- JavaScript example: `examples/javascript`
- Python examples: `examples/python`
- Rust examples: `examples/rust`

### Mesh quick example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{ Renderer, Pass, Shader, Mesh, Vertex, VertexValue };

let renderer = Renderer::new();
let target = renderer.create_texture_target([256, 256]).await?;

let wgsl = r#"
struct VertexOutput { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VertexOutput;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VertexOutput) -> @location(0) vec4<f32> { return vec4<f32>(0.2, 0.8, 0.2, 1.0); }
"#;

let shader = Shader::new(wgsl)?;
let pass = Pass::from_shader("mesh", &shader);

let mut mesh = Mesh::new();
mesh.add_vertices([
  Vertex::new([-0.5, -0.5, 0.0]),
  Vertex::new([ 0.5, -0.5, 0.0]),
  Vertex::new([ 0.0,  0.5, 0.0]),
]);
// Instance properties matched by name and type
mesh.add_instance(Vertex::new([0.0, 0.0]).set("offset", [0.0, 0.0]));

pass.add_mesh(&mesh);
renderer.render(&pass, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
