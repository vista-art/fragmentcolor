# Mesh::from_vertices

Create a mesh from an iterator of Vertex values.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex, Position};
let m = Mesh::from_vertices([
    Vertex::from_position(Position::Pos2([0.0, 0.0])),
    Vertex::from_position(Position::Pos2([1.0, 0.0])),
    Vertex::from_position(Position::Pos2([0.0, 1.0])),
]);
```