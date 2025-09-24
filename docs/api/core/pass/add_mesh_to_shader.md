# Pass::add_mesh_to_shader

Attach a Mesh to a specific Shader in this Pass. This forwards to `shader.add_mesh(mesh)` and returns the same Result.

## Example

```rust
use fragmentcolor::{Pass, Shader, Mesh, Vertex};

let shader = Shader::default();
let pass = Pass::from_shader("p", &shader);

let mut mesh = Mesh::new();
mesh.add_vertex(Vertex::new([0.0, 0.0]));

pass.add_mesh_to_shader(&mesh, &shader).expect("mesh is compatible");
```
