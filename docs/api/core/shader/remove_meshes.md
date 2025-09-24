# Shader::remove_meshes

Remove multiple meshes from this Shader.

## Example

```rust
use fragmentcolor::{Pass, Shader, Mesh, Vertex};

let shader = Shader::default();
let pass = Pass::from_shader("p", &shader);

let mut m1 = Mesh::new();
m1.add_vertex(Vertex::new([0.0, 0.0]));
let mut m2 = Mesh::new();
m2.add_vertex(Vertex::new([0.5, 0.0]));

shader.add_mesh(&m1);
shader.add_mesh(&m2);

shader.remove_meshes([&m1, &m2]);
```