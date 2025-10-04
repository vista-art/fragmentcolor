# Shader::remove_mesh

Remove a single Mesh previously attached to this Shader.
If the Mesh is attached multiple times, removes the first match.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Shader, Mesh};

let shader = Shader::new(r#"
  struct VOut { @builtin(position) pos: vec4<f32> };
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>) -> VOut {
    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    return out;
  }
  @fragment
  fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.0,0.0,0.0,1.0); }
"#)?;

let mesh = Mesh::new();
mesh.add_vertex([0.0, 0.0]);
shader.add_mesh(&mesh)?;

// Detach the mesh
shader.remove_mesh(&mesh);
# Ok(())
# }
```
