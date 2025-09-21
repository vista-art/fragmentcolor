# Vertex::create_instance

Create an Instance from this Vertex by cloning all of its properties

## Example

```rust
use fragmentcolor::mesh::Vertex;
let v = Vertex::new([0.0, 0.0]);
let inst = v.create_instance();
# _ = inst;
```
