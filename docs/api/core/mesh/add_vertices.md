# Mesh::add_vertices

Add many vertices to the mesh.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex, Position};
let mut m = Mesh::new();
m.add_vertices([
  Vertex::from_position(Position::Pos2([0.0, 0.0])),
  Vertex::from_position(Position::Pos2([1.0, 0.0]))
]);
```