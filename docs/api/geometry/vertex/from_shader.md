# Vertex::from_shader(shader: Shader)

Planned API (draft)

Construct a Vertex skeleton using reflection from the provided Shader. The returned Vertex contains:
- A position attribute with a format compatible with your intended input (defaults to vec2 or vec3 based on construction conventions).
- Property location assignments (name → @location(N)) pre-seeded from the shader’s vertex inputs.

Notes
- This does not assign property values; it only establishes the layout. Chain `set(...).at(...)` or `with(...)` calls to provide values.
- Location precedence at render time remains: explicit location (instance first, then vertex) → name fallback.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Shader};
use fragmentcolor::mesh::Vertex;

let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(1) offset: vec2<f32> };
@vertex fn vs_main(@location(0) position: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(position.xy + offset, position.z, 1.0);
  return out;
}
@fragment fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
"#;

let shader = Shader::new(wgsl)?;
// Planned helper: seeds property-to-location mapping from reflection
let mut v = Vertex::from_shader(&shader);
// Now attach values; locations were pre-seeded by reflection
v.set("offset", [0.0f32, 0.0]).at(1);
# Ok(())
# }
```