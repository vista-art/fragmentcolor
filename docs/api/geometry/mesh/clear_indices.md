# Mesh::clear_indices

Drop the user-supplied index buffer and return to auto-derived indexing.

After `set_indices` the mesh draws vertices in insertion order using the
indices you provided. `clear_indices` reverses that: the next render
re-runs the vertex dedup pass and rebuilds the index buffer from the
deduplicated unique set, the same way a freshly built mesh behaves.

## Example

```rust
use fragmentcolor::{Mesh, Vertex};

let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::new([-0.5, -0.5]),
    Vertex::new([ 0.5, -0.5]),
    Vertex::new([ 0.0,  0.5]),
]);
mesh.set_indices([0, 1, 2]);
mesh.clear_indices(); // back to auto-derived dedup
```
