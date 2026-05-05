# Vertex::create_instance

Create an `Instance` that inherits this vertex's attributes. Use this
when you want a starting point for an instance whose per-instance values
mostly match the source vertex, then call `set` on the result to change
the fields that differ.

## Example

```rust
use fragmentcolor::mesh::Vertex;
let v = Vertex::new([0.0, 0.0]);
let inst = v.create_instance();
# _ = inst;
```
