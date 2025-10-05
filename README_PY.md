# FragmentColor for Python

[FragmentColor](https://fragmentcolor.org) is a cross‑platform GPU programming library implemented in Rust and wgpu, with Python bindings via PyO3.

This README is specific to the PyPI package. For Rust usage, see the repository README.md. For JavaScript, see README_JS.md.

- Documentation: https://fragmentcolor.org/welcome
- API Reference: https://fragmentcolor.org/api

## Install

```bash
pip install fragmentcolor rendercanvas glfw
```

## Quick start

```python
from fragmentcolor import Renderer, Shader, Pass, Frame
from rendercanvas.auto import RenderCanvas, loop

# Initializes a renderer and a target compatible with the given canvas
canvas = RenderCanvas(size=(800, 600))
renderer = Renderer()
target = renderer.create_target(canvas)

# You can pass the shader as a source string, file path, or URL:
circle = Shader("./path/to/circle.wgsl")
triangle = Shader("https://fragmentcolor.org/shaders/triangle.wgsl")
shader = Shader("""
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
""")

# The library binds and updates the uniforms automatically
shader.set("my_struct.my_field", [0.1, 0.8, 0.9])
shader.set("my_vec2", [1.0, 1.0])

# One shader is all you need to render
renderer.render(shader, target)

# But you can also combine multiple shaders in a render Pass
rpass = Pass("single pass")
rpass.add_shader(circle)
rpass.add_shader(triangle)
rpass.add_shader(shader)
renderer.render(rpass, target)

# Finally, you can combine multiple passes in a Frame
frame = Frame()
frame.add_pass(rpass)
frame.add_pass(Pass("GUI pass"))
renderer.render(frame, target)

# To animate, simply update the uniforms in a loop
@canvas.request_draw
def animate():
    circle.set("position", [0.0, 0.0])
    renderer.render(frame, target)

loop.run()
```

## Python binding (local dev)

```bash
# Quick run helper: build wheel into dist/, create venv, and run an example
./run_py main      # or: ./run_py multiobject | ./run_py headless

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

- Swift & Kotlin bindings are not supported yet, but planned.

---

Note on generation: this README_PY.md is generated from this template (README_PY.tpl.md) and the repository README.md examples by the build script. Do not edit the generated README_PY.md directly.
