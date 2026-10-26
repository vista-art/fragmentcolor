# Vertex::new

Construct a `Vertex` from a 2D or 3D position. Set additional attributes
(`uv`, `color`, custom keys) with `set` to match the per-vertex inputs
your shader declares.

A 1-component position becomes 2D with `y` at 0. A 4-component position
uploads as 3D: the mesh drops the 4th component.

In JavaScript, Python, Swift and Kotlin, a position of another length, or a number
that is NaN, infinite or past the 32-bit float range, is an error that names the call.

## Example

```rust
use fragmentcolor::mesh::Vertex;
let v = Vertex::new([0.0, 0.0]);
```
