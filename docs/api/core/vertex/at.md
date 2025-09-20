# Vertex::set(...).at(index: u32)

Planned API (draft)

Pin the most recently `set(key, value)` property to a specific shader location `@location(index)`. Returns a mutable reference to the original `Vertex` so you can continue chaining additional `set` or `with` calls.

Behavior
- Overrides the auto-assigned location for the last `set` call.
- Calling `at(index)` multiple times for the same key updates the stored location.

## Example

```rust
use fragmentcolor::mesh::Vertex;

let mut v = Vertex::new([0.0, 0.0]);

v.set("offset", [0.0f32, 0.0])
 .at(1) // pin offset at @location(1)
 .set("tint", [1.0f32, 0.0, 0.0, 1.0])
 .at(2); // pin tint at @location(2)
```
