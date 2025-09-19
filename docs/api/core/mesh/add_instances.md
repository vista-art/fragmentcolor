# Mesh::add_instances

Add many instances to the mesh.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex};
let mut m = Mesh::new();
m.add_instances([
  Vertex::from([0.0, 0.0]),
  Vertex::from([1.0, 1.0]),
]);
```