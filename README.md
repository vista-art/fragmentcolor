# FragmentColor

[FragmentColor](https://fragmentcolor.org) is a cross-platform GPU programming library implemented in Rust and [wgpu](https://wgpu.rs).

It has bindings for **JavaScript** (WASM), **Python**, and draft support for **Swift** and **Kotlin**.
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

- JavaScript (Web): published to npm as `fragmentcolor`

```bash
npm install fragmentcolor
# or
pnpm add fragmentcolor
# or
yarn add fragmentcolor
```

- Python: published to PyPI as `fragmentcolor`

```bash
pip install fragmentcolor rendercanvas glfw
```

- Rust: add the crate to your Cargo.toml

```toml
[dependencies]
fragmentcolor = "0.10.7"
```

## Quick start

### JavaScript

```js
import init, { Renderer, Shader } from "fragmentcolor";

async function start() {
  await init(); // initialize the WASM module
  const canvas = document.getElementById("my-canvas");

  const renderer = new Renderer();
  const target = await renderer.createTarget(canvas);

  const shader = new Shader("https://fragmentcolor.org/shaders/circle.wgsl");
  shader.set("resolution", [canvas.width, canvas.height]);
  shader.set("circle.radius", 0.05);
  shader.set("circle.color", [1.0, 0.0, 0.0, 0.8]);

  function animate() {
    // update uniforms and render
    renderer.render(shader, target);
    requestAnimationFrame(animate);
  }
  animate();
}
start();
```

### Python

```python
from fragmentcolor import Renderer, Shader, Pass, Frame
from rendercanvas.auto import RenderCanvas, loop

canvas = RenderCanvas(size=(800, 600))
renderer = Renderer()
target = renderer.create_target(canvas)

circle = Shader("https://fragmentcolor.org/shaders/circle.wgsl")
circle.set("resolution", [800, 600])
circle.set("circle.radius", 200.0)
circle.set("circle.color", [1.0, 0.0, 0.0, 0.8])

rpass = Pass("single pass")
rpass.add_shader(circle)

frame = Frame()
frame.add_pass(rpass)

@canvas.request_draw
def animate():
    renderer.render(frame, target)

loop.run()
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
use fragmentcolor::{Renderer, Pass, Shader};
use fragmentcolor::mesh::{Mesh, Vertex, VertexValue};

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
mesh.add_instance(Vertex::new([0.0, 0.0]).with("offset", [0.0, 0.0]));

pass.add_mesh(&mesh);
renderer.render(&pass, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
