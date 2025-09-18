# Shader::set(key: string, value: any)

Sets the value of the uniform identified by the given key.

If the key does not exist or the value format is incorrect, the `set` method throws an exception. The shader remains valid, and if the exception is caught, the shader can still be used with the renderer.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader};
let r = Renderer::new();
let shader = Shader::new(r#"
@group(0) @binding(0) var<uniform> resolution: vec2<f32>;

struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
  var out: VOut;
  out.pos = vec4<f32>(p[i], 0., 1.);
  return out;
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
"#)?;

// Set scalars/vectors on declared uniforms
shader.set("resolution", [800.0, 600.0])?;
# Ok(())
# }
```
