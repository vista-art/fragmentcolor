# Vertex::create_instance

Create an Instance from this Vertex by cloning all of its properties; the position is copied under `position2` or `position3`.

## Example

```rust
use fragmentcolor::mesh::Vertex;
let v = Vertex::new([0.0, 0.0]);
let _inst = v.create_instance();
```
