# Vertex::with_color

Attach a color to the vertex.

## Example

```rust
use fragmentcolor::mesh::{Vertex, Position};
let v = Vertex::from_position(Position::Pos3([0.0, 0.0, 0.0])).with_color([1.0, 0.0, 0.0, 1.0]);
```