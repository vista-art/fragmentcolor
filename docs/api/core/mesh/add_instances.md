# Mesh::add_instances

Add many instances to the mesh.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex, Position};
let mut m = Mesh::new();
m.add_instances([
  Vertex::from_position(Position::Pos2([0.0, 0.0])),
  Vertex::from_position(Position::Pos2([1.0, 1.0])),
]);
```