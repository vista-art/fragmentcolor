# Vertex::new

Construct a `Vertex` from a 2D or 3D position. Set additional attributes
(`uv`, `color`, custom keys) with `set` to match the per-vertex inputs
your shader declares.

## Example

```rust
use fragmentcolor::mesh::Vertex;
let v = Vertex::new([0.0, 0.0]);
```
