# Pass::add_mesh

Attach a Mesh to this Pass. The Renderer will draw the mesh for each attached Shader.

## Example

```rust
use fragmentcolor::{Pass, Shader};
use fragmentcolor::mesh::{Mesh, Vertex};

let shader = Shader::default();
let pass = Pass::from_shader("p", &shader);
let mut mesh = Mesh::new();
mesh.add_vertex(Vertex::new([0.0, 0.0]));
pass.add_mesh(&mesh);
```
