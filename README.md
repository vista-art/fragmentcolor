# FragmentColor

[FragmentColor](https://fragmentcolor.org) is a cross-platform GPU programming library implemented in Rust and [wgpu](https://wgpu.rs).

It is implemented in **Rust**, with bindings for **Javascript**, **Python**, **Swift**, and **Kotlin**,
and targets each platform's native graphics API: **Vulkan**, **Metal**, **DirectX**, **OpenGL**, **WebGL**, or **WebGPU**.\
See [Platform Support](#platform-support) for details.

The API encourages a simple shader composition workflow. You can use **WGSL** or **GLSL** shaders
for visual consistency across platforms, while avoiding the verbosity of modern GPU APIs.

**We strive to remove the complexity without sacrificing control**. Because of the composition priomitives, you can
build a highly customized render graph with multiple render passes.

Check the [Documentation](/welcome) and the [API Reference](/api) for more information.

## Example

From a given shader source, our library will:

- parse the shader
- compile/reload it at runtime
- create the Uniform bindings in your platform's native graphics API
- expose them with the dot notation.

Consider this simple WGSL example:

```rust
// @vertex ommited for brevity

struct MyStruct {
    field: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> my_struct: MyStruct;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(my_struct.rgb, 1.0);
}
```

### Example usage (Python)

```python
import fragmentcolor as fc

# Parse the source and binds uniforms automatically
shader = fc.Shader("my_shader.wgsl")
shader.set("my_uniform.field", [1.0, 1.0, 1.0])

# Render to image
renderer = fc.Renderer()
img = renderer.render_image(shader)
```

### Example usage (Javascript)

```javascript
import { Shader, Renderer, Target, FragmentColor } from "fragmentcolor";

let canvas = document.getElementById("my-canvas");
const resolution = [canvas.width, canvas.heigth];

[renderer, target] = FragmentColor.init(canvas);

const shader = new Shader("https://example.com/circle.wgsl");
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

There are no published distributions at this moment.

You can build it locally with [maturin](https://www.maturin.rs/installation.html):

```bash
pipx install maturin
maturin develop
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
