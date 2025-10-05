# Pass::add_mesh_to_shader

Attach a Mesh to a specific Shader in this Pass. This forwards to `shader.add_mesh(mesh)` and returns the same Result.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Shader, Mesh, Vertex, Pass};

let mesh = Mesh::new();
mesh.add_vertex(Vertex::new([0.0, 0.0]));
let shader = Shader::new(r#"
  struct VOut { @builtin(position) pos: vec4<f32> };
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>) -> VOut {
    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    return out;
  }
  @fragment
  fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
"#)?;

let pass = Pass::from_shader("pass", &shader);
pass.add_mesh_to_shader(&mesh, &shader)?;

# Ok(())
# }
```
