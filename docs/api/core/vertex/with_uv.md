# Vertex::with_uv

Attach a UV to the vertex.

## Example

```rust
use fragmentcolor::mesh::{Vertex, Position};
let v = Vertex::from_position(Position::Pos3([0.0, 0.0, 0.0])).with_uv([0.0, 1.0]);
```