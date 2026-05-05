# Mesh::add_vertex

Append a single vertex to the mesh. Pass an array literal for the
position (`[x, y]` or `[x, y, z]`); for vertices that carry `uv`, `color`,
or other per-vertex attributes, build a [Vertex](https://fragmentcolor.org/api/geometry/vertex)
first and pass it in.

## Example

```rust
use fragmentcolor::mesh::{Mesh};
let mut m = Mesh::new();
m.add_vertex([0.0, 0.0]);
```
