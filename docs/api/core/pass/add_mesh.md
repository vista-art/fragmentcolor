# Pass::add_mesh

Attach a Mesh to this Pass.

- The mesh is attached to the last shader in this Pass that accepts it.
- Each shader checks the mesh against its vertex inputs, as
  [`Shader::add_mesh`](https://fragmentcolor.org/api/core/shader#add_mesh) does.
  A shader that rejects the mesh leaves no type on it, and the next one tries.
- Returns `Result<(), PassError>`. When no shader accepts the mesh, the error is
  `PassError::NoCompatibleShader` with the reason the last shader gave, and the
  mesh is not attached.

A Pass with no shaders has nothing to attach the mesh to and returns
`PassError::NoShader`. No default shader is created.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Pass, Shader, Mesh};

let mesh = Mesh::new();
mesh.add_vertex([0.0, 0.0]);

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

pass.add_mesh(&mesh)?;

# Ok(())
# }
```
