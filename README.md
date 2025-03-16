# FragmentColor

[FragmentColor](https://fragmentcolor.org) is a cross-platform GPU programming library implemented in Rust and [wgpu](https://wgpu.rs).

It is implemented in **Rust**, with bindings for **Javascript**, **Python**, **Swift**, and **Kotlin**,
and targets each platform's native graphics API: **Vulkan**, **Metal**, **DirectX**, **OpenGL**, **WebGL**, or **WebGPU**.\
See [Platform Support](#platform-support) for details.

The API encourages a simple shader composition workflow. You can use **WGSL** or **GLSL** shaders
for visual consistency across platforms, while avoiding the verbosity of modern GPU APIs.

**We strive to remove the complexity without sacrificing control**. Because of the composition primitives, you can
build a highly customized render graph with multiple render passes.

Check the [Documentation](/welcome) and the [API Reference](/api) for more information.

> [!WARNING] This library is its early days of development
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

> [!WARNING] NOTE
>
> Pip Package is currently only available for MacOS (Apple Silicon).\
> You can also [build it locally](#target-desktop-python-module) for your platform.

```bash
pip install fragmentcolor glfw rendercanvas
```

```python
from fragmentcolor import FragmentColor as fc, Shader
from rendercanvas.auto import RenderCanvas, loop

canvas = RenderCanvas(size=(800, 600))
renderer, target = fc.init(canvas)

# You can pass the shader as a source string, file path, or URL:
circle = Shader("./path/to/circle.wgsl")
triangle = Shader("https://fragmentcolor.org/shaders/circle.wgsl")
my_shader = Shader("""
  // @vertex ommited for brevity

  struct MyStruct {
      field: vec3<f32>,
  }

  @group(0) @binding(0)
  var<uniform> my_struct: MyStruct;

  @group(0) @binding(1) var<uniform> my_vec2: vec2<f32>;

  @fragment
  fn fs_main() -> @location(0) vec4<f32> {
      return vec4<f32>(my_struct.rgb, 1.0);
  }
""")

# The library binds and updates the uniforms automatically:
my_shader.set("my_uniform.field", [1.0, 1.0, 1.0])
my_shader.set("my_vec2", [1.0, 1.0])

@canvas.request_draw
def animate():
    renderer.render(shader, target)

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

- Textures and Samplers are currently not supported, but are also planned.

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
