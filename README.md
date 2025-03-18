# FragmentColor

[FragmentColor](https://fragmentcolor.org) is a cross-platform GPU programming library implemented in Rust and [wgpu](https://wgpu.rs).

It has bindings for **Javascript**, **Python**, **Swift**, and **Kotlin**
and targets each platform's native graphics API: **Vulkan**, **Metal**, **DirectX**, **OpenGL**, **WebGL**, or **WebGPU**.\
See [Platform Support](#platform-support) for details.

The API encourages a simple shader composition workflow. You can use **WGSL** or **GLSL** shaders
for visual consistency across platforms, while avoiding the verbosity of modern GPU APIs.

**We strive to remove the complexity without sacrificing control**. Because of the composition primitives, you can
build a highly customized render graph with multiple render passes.

Check the [Documentation](/welcome) and the [API Reference](/api) for more information.

> ⚠️ **This library is its early days of development**
>
> The API is subject to frequent changes in minor versions. Documentation is not always in sync.
>
> Check the [Roadmap](/ROADMAP.md) and [Changelog](/CHANGELOG.md) on [GitHub](https://github.com/vista-art/fragmentcolor) to stay tuned on the latest updates.

## Example

From a given shader source, our library will:

- parse the shader
- compile/reload it at runtime
- create the Uniform bindings in your platform's native graphics API
- expose them with the dot notation.

### Example usage (Python)

```bash
pip install fragmentcolor glfw rendercanvas
```

```python
from fragmentcolor import FragmentColor as fc, Shader, Pass, Frame
from rendercanvas.auto import RenderCanvas, loop

# Initializes a renderer and a target compatible with the given canvas
canvas = RenderCanvas(size=(800, 600))
renderer, target = fc.init(canvas)

# You can pass the shader as a source string, file path, or URL:
circle = Shader("./path/to/circle.wgsl")
triangle = Shader("https://fragmentcolor.org/shaders/triangle.wgsl")
my_shader = Shader("""
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
my_shader.set("my_struct.my_field", [0.1, 0.8, 0.9])
my_shader.set("my_vec2", [1.0, 1.0])

# One shader is all you need to render
renderer.render(shader, target)

# But you can also combine multiple shaders in a render Pass
rpass = Pass("single pass")
rpass.add_shader(circle)
rpass.add_shader(triangle)
rpass.add_shader(my_shader)
renderer.render(rpass, target)

# Finally, you can combine multiple passes in a Frame
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

### Example usage (Javascript)

```javascript
import { Shader, Renderer, Target, FragmentColor } from "fragmentcolor";

let canvas = document.getElementById("my-canvas");
const resolution = [canvas.width, canvas.heigth];

[renderer, target] = FragmentColor.init(canvas);

const shader = new Shader("https://fragmentcolor.org/shaders/circle.wgsl");
shader.set("resolution", resolution);
shader.set("circle.radius", 0.05);
shader.set("circle.color", [1.0, 0.0, 0.0, 0.8]);

const renderer = new Renderer();

function animate() {
  shader.set("circle.position", [mouseX, mouseY]);
  renderer.render(shader, target);

  requestAnimationFrame(animate);
}
animate();
```

## Limitations

- The current version of this library **always use a fullscreen triangle for every shader**. Support for custom geometries and instanced rendering is planned.

- In Python, we depend on [rendercanvas](https://github.com/pygfx/rendercanvas) adapter to support multiple window libraries. Direct support for other libraries is planned.

- Textures and Samplers are currently not supported, but are also planned.

- Javascript, Swift, and Kotlin are currently WIP.

## Running this project

### Target: Desktop (Rust library)

For Rust, check the examples folder and run them with:

```bash
cargo run --example circle
cargo run --example triangle
cargo run --example multiobject
cargo run --example multipass
```

### Target: Desktop (Python module)

**NOTE:** Pip Package currently only available for MacOS (Apple Silicon)

```bash
pip install fragmentcolor glfw rendercanvas
```

Alternativaly, You can build it locally with [maturin](https://www.maturin.rs/installation.html):

```bash
pipx install maturin
maturin develop
pip install glfw rendercanvas
```

The built library is located in `platforms/python/fragmentcolor`

```bash
cd platforms/python/fragmentcolor
python3 main.py
```

### Target: Web browser (WASM module)

- TBD

### Target: iOS (Swift library)

- TBD

### Target: Android (Kotlin library)

- TBD

## Platform support

Platform support is the same as upstream [wgpu](https://github.com/gfx-rs/wgpu):

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
