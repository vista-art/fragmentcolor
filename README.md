# FragmentColor

FragmentColor is a **Cross-Platform GPU Programming Library** based on Rust and [wgpu](https://github.com/gfx-rs/wgpu).

It provides a simple shader composition API for **Javascript**, **Python**, **Swift**, **Kotlin**,
so you can use **WGSL** or **GLSL** shaders as the source of truth for visual consistency across platforms.

Our library removes all the boilerplate needed to make a modern rendering pipeline work. Your shader will target the platform's native graphics API: Vulkan, Metal, DirectX, OpenGL, WebGL, or WebGPU.

See [Platform Support](#platform-support) for details.

**TL;DR** From a given **shader source**, our library will:

- parse the shader
- compile/reload it at runtime
- create the Uniform bindings in your platform's native graphics API
- expose them with the dot notation.

## Example

Consider this simple WGSL shader source:

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
import { Shader, Renderer, Target } from "fragmentcolor";

let canvas = document.getElementById("my-canvas");
const resolution = [canvas.width, canvas.heigth];

const shader = new Shader("circle.wgsl");
shader.set("resolution", resolution);
shader.set("circle.radius", 0.05);
shader.set("circle.color", [1.0, 0.0, 0.0, 0.8]);

const renderer = new Renderer();

function animate() {
  shader.set("circle.position", [mouseX, mouseY]);
  renderer.render(shader, canvas);

  requestAnimationFrame(animate);
}
animate();
```

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

## Building this project

### Running examples

```bash
cargo run --example circle
cargo run --example triangle
```

### Target: Desktop or Server/CI (Rust library)

- TBD

### Target: Desktop or Server/CI (Python module)

- TBD

### Target: Web browser (WASM module)

- TBD

### Target: iOS (Swift library)

- TBD

### Target: Android (Kotlin library)

- TBD
