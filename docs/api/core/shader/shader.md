# Shader

The [Shader](https://fragmentcolor.org/api/core/shader) object is the main building block in [FragmentColor](https://fragmentcolor.org).

It takes a WGSL or GLSL shader source as input, parses it, validates it, and exposes the uniforms as keys.

To draw your shader, you must use your [Shader](https://fragmentcolor.org/api/core/shader) instance as input to a [Renderer](https://fragmentcolor.org/api/core/renderer).

You can compose [Shader](https://fragmentcolor.org/api/core/shader) instances into a [Pass](https://fragmentcolor.org/api/core/pass) object to create more complex rendering pipelines.

You can also create renderings with multiple Render Passes by using multiple [Pass](https://fragmentcolor.org/api/core/pass) instances to a [Frame](https://fragmentcolor.org/api/core/frame) object.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Shader, Renderer};

let shader = Shader::new(r#"
    @vertex
    fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
        var pos = array<vec2<f32>, 3>(
            vec2<f32>(-1.0, -1.0),
            vec2<f32>( 3.0, -1.0),
            vec2<f32>(-1.0,  3.0)
        );
        return vec4<f32>(pos[index], 0.0, 1.0);
    }

    @group(0) @binding(0)
    var<uniform> resolution: vec2<f32>;

    @fragment
    fn fs_main() -> @location(0) vec4<f32> {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red
    }
"#)?;

// Set the "resolution" uniform
shader.set("resolution", [800.0, 600.0])?;
let res: [f32; 2] = shader.get("resolution")?;

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
renderer.render(&shader, &target)?;

# assert_eq!(res, [800.0, 600.0]);
# assert!(shader.list_uniforms().len() >= 1);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
