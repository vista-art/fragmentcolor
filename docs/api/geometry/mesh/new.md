# Mesh::new

Create an empty mesh. Add vertices with `add_vertex` (or `add_vertices`)
and instances with `add_instance`, then attach the mesh to a shader to
draw it.

## Example

```rust
use fragmentcolor::mesh::Mesh;
let m = Mesh::new();
# _ = m;
```
