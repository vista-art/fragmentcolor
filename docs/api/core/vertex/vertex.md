# Vertex

A single vertex with a required position (2D or 3D) and optional properties like uv and color.

## Example

```rust
use fragmentcolor::mesh::Vertex;
let v = Vertex::new([0.0, 0.0, 0.0]).set("uv", [0.5, 0.5]);
```
