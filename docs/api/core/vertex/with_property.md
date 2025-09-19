# Vertex::with_property

Attach an arbitrary property to the vertex.

## Example

```rust
use fragmentcolor::mesh::{Vertex, VertexValue};
let v = Vertex::new([0.0, 0.0, 0.0]).with_property("weight", VertexValue::F32(1.0));
```
