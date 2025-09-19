# Vertex::create_instance

Create an Instance from this Vertex by cloning all of its properties; the position is copied under `position2` or `position3`.

## Example

```rust
use fragmentcolor::mesh::{Vertex, Position};
let v = Vertex::from_position(Position::Pos2([0.0, 0.0]));
let _inst = v.create_instance();
```