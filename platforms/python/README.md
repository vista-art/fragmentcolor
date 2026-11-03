# FragmentColor for Python

[FragmentColor](https://fragmentcolor.org) is a universal GPU abstraction for Rust, JavaScript, Python, Swift, and Kotlin. This package is the Python binding, built with PyO3.

This README is specific to the PyPI package. For Rust usage, see the repository README.md. For JavaScript, see `platforms/web/README.md` in the repository.

- Documentation: https://fragmentcolor.org/welcome
- API Reference: https://fragmentcolor.org/api

## Install

```bash
pip install fragmentcolor rendercanvas glfw
```

NumPy installs with the package as a required dependency. `TextureTarget.get_image()` returns its pixels as a NumPy array of shape `(height, width, 4)`, and `Renderer.create_texture` accepts one as input.

## Quick start

```python
import math
import time

from fragmentcolor import Renderer, Shader, Pass
from rendercanvas.auto import RenderCanvas, loop

# A renderer and a target that draws into the canvas
canvas = RenderCanvas(size=(800, 600))
renderer = Renderer()
target = renderer.create_target(canvas)

# A shader from WGSL source. Its uniforms are ready to set by name.
background = Shader("""
struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
}

struct MyStruct {
    my_field: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> my_struct: MyStruct;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let vertices = array(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.));
    return VertexOutput(vec4<f32>(vertices[index], 0.0, 1.0));
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(my_struct.my_field, 1.0);
}
""")
background.set("my_struct.my_field", [0.1, 0.8, 0.9])

# Library functions join your own source, which calls them.
# The package carries the library, so this works offline.
circle = Shader(["sdf2d/circle", """
@group(0) @binding(0)
var<uniform> center: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let vertices = array(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.));
    return vec4<f32>(vertices[index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    if circle(pos.xy - center, 100.0) > 0.0 {
        discard;
    }
    return vec4<f32>(1.0, 0.4, 0.2, 1.0);
}
"""])

# Passes render in order, and the second one keeps what the first drew.
# One shader on its own renders too: renderer.render(background, target).
circle_pass = Pass.from_shader("circle", circle)
circle_pass.load_previous()
passes = [Pass.from_shader("background", background), circle_pass]

# To animate, update the uniforms and render again on each frame
@canvas.request_draw
def animate():
    size = target.size()
    x = size.width / 2 + math.cos(time.time()) * 100
    circle.set("center", [x, size.height / 2])
    renderer.render(passes, target)
    canvas.request_draw()

loop.run()
```

## Python binding (local dev)

```bash
# Quick run helper: build wheel into dist/, create venv, and run an example
./fc run_py main      # or: ./fc run_py multiobject | ./fc run_py headless

# Manual alternative
pipx install maturin
maturin develop
pip install glfw rendercanvas
python examples/python/main.py
```

## Documentation & website

- Docs source of truth lives in docs/api and is referenced from code via `#[lsp_doc]`.
- Examples on method pages are sliced from the healthcheck scripts; no filesystem reads in docs.
- Doc examples follow async + pollster patterns on the Rust side and are converted to Python automatically.

## Platform support

Platform support is aligned with upstream wgpu:

| API    | Windows      | Linux/Android   | macOS/iOS | Web (wasm)  |
| ------ | ------------ | --------------- | --------- | ----------- |
| Vulkan | ✅           | ✅              | 🌋        |             |
| Metal  |              |                 | ✅        |             |
| DX12   | ✅           |                 |           |             |
| OpenGL | 🆗 (GL 3.3+) | 🆗 (GL ES 3.0+) | 📐        | 🆗 (WebGL2) |
| WebGPU |              |                 |           | ✅          |

✅ = First Class Support  
🆗 = Downlevel/Best Effort Support  
📐 = Requires the ANGLE translation layer (GL ES 3.0 only)  
🌋 = Requires the MoltenVK translation layer

## Limitations (planned features)

- See the ROADMAP and CHANGELOG on the [repository](https://github.com/vista-art/fragmentcolor) for planned features and known limitations.

---

Note on generation: this README.md is generated from README.tpl.md (same directory) and the repository README.md examples by the build script. Do not edit the generated README.md directly.
