# Mesh::add_instance

Add a single instance (any Vertex can be converted into an instance).

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex, Position};
let mut m = Mesh::new();
let v = Vertex::from_position(Position::Pos2([0.0, 0.0]));
m.add_instance(v);
```