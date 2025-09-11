# Shader::default()

Creates a new [Shader](https://fragmentcolor.org/api/core/shader) from the default shader source.

Useful for examples and quick demos. Includes a "**resolution**" uniform.

```wgsl
struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0));
}

@fragment
fn fs_main(pixel: VertexOutput) -> @location(0) vec4<f32> {
    let _dummy = resolution.x + resolution.y * 0.0;
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
```

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Shader};

let shader = Shader::default();

// The default shader includes a "resolution" uniform
shader.set("resolution", [800.0, 600.0])?;
let res: [f32; 2] = shader.get("resolution")?;

# assert!(shader.list_keys().len() >= 1);
# assert_eq!(res, [800.0, 600.0]);
# Ok(())
# }
```
