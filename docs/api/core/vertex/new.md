# Vertex::new

Create a Vertex at the origin (2D). This is a helper used in docs; prefer `from_position` in code.

## Example

```rust
use fragmentcolor::mesh::{Vertex, Position};
let v = Vertex::from_position(Position::Pos2([0.0, 0.0]));
```