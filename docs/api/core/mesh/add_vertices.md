# Mesh::add_vertices

Add many vertices to the mesh.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex};
let mut m = Mesh::new();
m.add_vertices([
  Vertex::new([0.0, 0.0]),
  Vertex::new([1.0, 0.0])
]);
```
