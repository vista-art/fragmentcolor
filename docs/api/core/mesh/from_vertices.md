# Mesh::from_vertices

Create a mesh from an iterator of Vertex values.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex};
let m = Mesh::from_vertices([
    Vertex::from([0.0, 0.0]),
    Vertex::from([1.0, 0.0]),
    Vertex::from([0.0, 1.0]),
]);
```