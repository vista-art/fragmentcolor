# Shader::remove_mesh

Remove a single Mesh previously attached to this Shader.
If the Mesh is attached multiple times, removes the first match.

## Example

```rust
use fragmentcolor::{Pass, Shader, Mesh, Vertex};

let shader = Shader::default();
let pass = Pass::from_shader("p", &shader);

let mut mesh = Mesh::new();
mesh.add_vertex(Vertex::new([0.0, 0.0]));
shader.add_mesh(&mesh);

// Detach the mesh
shader.remove_mesh(&mesh);
```