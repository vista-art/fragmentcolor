# Pass::add_mesh_to_shader

Attach a Mesh to a specific Shader in this Pass. This forwards to `shader.add_mesh(mesh)` and returns the same Result.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Shader, Mesh, Vertex, Pass};

let mut mesh = Mesh::new();
mesh.add_vertex([0.0, 0.0]);
let shader = Shader::from_mesh(&mesh);
let pass = Pass::from_shader("pass", &shader);

pass.add_mesh_to_shader(&mesh, &shader)?;

# Ok(())
# }
```
