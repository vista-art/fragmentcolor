# Mesh::add_instance

Add a single instance (any Vertex can be converted into an instance).

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex};
let mut m = Mesh::new();
let v = Vertex::new([0.0, 0.0]);
m.add_instance(v);
```
