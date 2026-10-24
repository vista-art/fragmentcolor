# Shader::validate_mesh

Validate that a Mesh is compatible with this Shader's vertex inputs.

- Checks that every `@location` input of the vertex entry point finds an attribute, and that every vertex and instance value fits the type the input declares.
- Matches each input in this order:
  1) by name, instance attributes first, then vertex attributes (`position` included)
  2) the position, at `@location(0)`
  3) by the attribute's own location, instance attributes first, then vertex attributes
- A number fills a float input unless the shader declares an integer type there. An integer input takes whole numbers that fit its type.
- Returns `Ok(())` when every input matches; returns the same error `Shader::add_mesh` would otherwise.

## Notes

- `Shader::add_mesh` and `Pass::add_mesh` run the same check. Call `validate_mesh` to test a mesh without attaching it.
- It changes nothing: the mesh isn't attached and its attributes take no type.
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
