# Vertex

A single vertex with a required position (2D or 3D) and optional properties like uv and color.

## Example

```rust
use fragmentcolor::mesh::{Vertex, Position};
let v = Vertex::from_position(Position::Pos3([0.0, 0.0, 0.0])).with_uv([0.5, 0.5]);
```