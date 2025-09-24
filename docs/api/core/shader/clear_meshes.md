# Shader::clear_meshes

Remove all meshes attached to this Shader.

## Example

```rust
use fragmentcolor::{Pass, Shader};
use fragmentcolor::mesh::{Mesh, Vertex};

let shader = Shader::default();
let pass = Pass::from_shader("p", &shader);

let mut mesh = Mesh::new();
mesh.add_vertex(Vertex::new([0.0, 0.0]));
shader.add_mesh(&mesh);

// Clear all
shader.clear_meshes();
```