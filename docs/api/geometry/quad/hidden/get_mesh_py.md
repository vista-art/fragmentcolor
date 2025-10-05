# Quad::get_mesh()

Gets the Mesh built by this Quad.

## Example

```rust
use fragmentcolor::{Quad, Mesh};

let quad = Quad::new([-0.5, -0.5], [0.5, 0.5]);
let mesh: Mesh = quad.get_mesh();

# _ = mesh;
```
