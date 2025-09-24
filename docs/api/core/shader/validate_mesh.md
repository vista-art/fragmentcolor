# Shader::validate_mesh

Validate that a Mesh is compatible with this Shader’s vertex inputs.

- Checks presence and type for all @location(...) inputs of the vertex entry point.
- Matches attributes in the following order:
  1) Instance attributes by explicit @location index (if the mesh has instances)
  2) Vertex attributes by explicit @location index (position is assumed at @location(0))
  3) Fallback by name (tries instance first, then vertex)
- Returns Ok(()) when all inputs are matched with a compatible wgpu::VertexFormat; returns an error otherwise.

Notes
- If the Shader has no @location inputs, any Mesh is considered compatible (the mesh is ignored at draw-time).
- This method does not allocate GPU buffers; it inspects CPU-side vertex/instance data only.

## Example

```rust
use fragmentcolor::{Shader, Pass};
use fragmentcolor::mesh::{Mesh, Vertex};

let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@location(0) pos: vec3<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 1.0);
  return out;
}
@fragment fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
"#;
let shader = Shader::new(wgsl).unwrap();
let pass = Pass::from_shader("p", &shader);

let mut mesh = Mesh::new();
mesh.add_vertices([
  Vertex::new([-0.5, -0.5, 0.0]),
  Vertex::new([ 0.5, -0.5, 0.0]),
  Vertex::new([ 0.0,  0.5, 0.0]),
]);

shader.validate_mesh(&mesh).unwrap(); // Ok
pass.add_mesh(&mesh).expect("mesh is compatible");
```