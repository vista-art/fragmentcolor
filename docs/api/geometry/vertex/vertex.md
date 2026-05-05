# Vertex

A single vertex: a required 2D or 3D position plus any per-vertex
attributes your shader declares (commonly `uv` and `color`, but any key
the shader's vertex stage reads is fair game). Build vertices and feed
them into a [Mesh](https://fragmentcolor.org/api/geometry/mesh) to draw.

## Example

```rust
use fragmentcolor::mesh::Vertex;
let v = Vertex::new([0.0, 0.0, 0.0]).set("uv", [0.5, 0.5]);
```
