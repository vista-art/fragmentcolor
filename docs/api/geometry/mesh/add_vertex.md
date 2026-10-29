# Mesh::add_vertex

Append a single vertex to the mesh. Pass an array literal for the
position (`[x, y]` or `[x, y, z]`); for vertices that carry `uv`, `color`,
or other per-vertex attributes, build a [Vertex](https://fragmentcolor.org/api/geometry/vertex)
first and pass it in.

A vertex added after the mesh attaches to a shader holds every attribute that
shader reads. `Renderer.render` returns `MeshError::InvalidVertex` for one
that lacks an attribute, naming the attribute and the vertex.

## Example

```rust
use fragmentcolor::mesh::{Mesh};
let mut m = Mesh::new();
m.add_vertex([0.0, 0.5]);
m.add_vertex([-0.5, -0.5]);
m.add_vertex([0.5, -0.5]);
```
