# Pass::add_mesh

Attach a Mesh to this Pass.

- The mesh is attached to the last shader previously added to this Pass.
- Validates compatibility with that shader’s vertex inputs.
- Returns Result<(), ShaderError>; on error, the mesh is not attached.

If a Shader wasn't provided earlier, FragmentColor will create a default one.

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
