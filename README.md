# FragmentColor

[FragmentColor](https://fragmentcolor.org) is a cross-platform GPU programming library implemented in Rust and [wgpu](https://wgpu.rs).

It has bindings for **JavaScript** (WASM), **Python**, and draft support for **Swift** and **Kotlin**.
It targets each platform's native graphics API: **Vulkan**, **Metal**, **DirectX**, **OpenGL**, **WebGL**, and **WebGPU**.
See [Platform Support](#platform-support) for details.

The API encourages a simple shader composition workflow. You can use **WGSL** or **GLSL** shaders
for visual consistency across platforms, while avoiding the verbosity of modern GPU APIs.

Check the website for the Getting Started guide and full reference:
- Documentation: https://fragmentcolor.org/welcome
- API Reference: https://fragmentcolor.org/docs/api

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
pip install fragmentcolor
# many examples also use
pip install rendercanvas glfw
```

- Rust: add the crate to your Cargo.toml

```toml
[dependencies]
fragmentcolor = "0.10.7"
```

## Quick start

### JavaScript (Web)

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

### Python (Desktop)

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

## Limitations (current focus)

- Fullscreen triangle rendering only; geometry/instancing planned.
- Textures and samplers are WIP.
- Swift & Kotlin bindings are drafts.

## Development

- JavaScript example: `examples/javascript`
- Python examples: `examples/python`
- Rust examples: `examples/rust`

Useful commands:

```bash
# format & lint (Rust)
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings

# run tests
cargo test

# build docs site
pnpm --dir docs/website install
pnpm --dir docs/website build
```
