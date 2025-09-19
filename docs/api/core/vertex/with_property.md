# Vertex::with_property

Attach an arbitrary property to the vertex.

Any property added here can be consumed by a shader vertex parameter as long as the
names and types match (e.g., a WGSL parameter named `@location(1) offset: vec2<f32>`
will bind a property key `"offset"` with type Float32x2).

## Example

```rust
use fragmentcolor::mesh::{Vertex, VertexValue};
let v = Vertex::new([0.0, 0.0, 0.0]).with_property("weight", VertexValue::F32(1.0));
```
