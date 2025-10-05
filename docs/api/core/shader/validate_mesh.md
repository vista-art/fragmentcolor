# Shader::validate_mesh

Validate that a Mesh is compatible with this Shader's vertex inputs.

- Checks presence and type for all @location(...) inputs of the vertex entry point.
- Matches attributes in the following order:
  1) Instance attributes by explicit @location index (if the mesh has instances)
  2) Vertex attributes by explicit @location index (position is assumed at @location(0))
  3) Fallback by name (tries instance first, then vertex)
- Returns Ok(()) when all inputs are matched with a compatible wgpu::VertexFormat; returns an error otherwise.

## Notes

- This method is called automatically when adding a Mesh to a Shader or Pass, so you usually don't need to call it manually.
- If the Shader has no @location inputs (fullscreen/builtin-only), attaching a Mesh is rejected.
- This method does not allocate GPU buffers; it inspects CPU-side vertex/instance data only.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Shader, Pass, Mesh};

let shader = Shader::new(r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@location(0) pos: vec3<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 1.0);
  return out;
}
@fragment fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
"#)?;
let pass = Pass::from_shader("p", &shader);

let mesh = Mesh::new();
mesh.add_vertices([
  [-0.5, -0.5, 0.0],
  [ 0.5, -0.5, 0.0],
  [ 0.0,  0.5, 0.0],
]);

shader.validate_mesh(&mesh)?; // Ok
pass.add_mesh(&mesh)?;

# Ok(())
# }
```
