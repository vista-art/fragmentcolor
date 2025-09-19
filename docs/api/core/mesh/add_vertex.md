# Mesh::add_vertex

Add a single vertex to the mesh.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex};
let mut m = Mesh::new();
m.add_vertex(Vertex::from([0.0, 0.0]));
```