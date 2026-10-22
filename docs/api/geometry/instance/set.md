# Instance::set

Set a per-instance attribute. `set` returns a new `Instance` with the
attribute added, so chain the calls and keep the result.

The first `set` for a key gives it the next free instance `@location`,
in call order; setting the same key again keeps its location. Plain
numbers and number lists are floats in every language. Pass a
[VertexValue](https://fragmentcolor.org/api/geometry/vertexvalue) for an
integer attribute.

## Example

```rust
use fragmentcolor::mesh::Instance;

let instance = Instance::new()
    .set("offset", [0.5, 0.0])
    .set("tint", [1.0, 0.0, 0.0, 1.0]);
# _ = instance;
```
