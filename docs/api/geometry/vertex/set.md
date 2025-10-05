# Vertex::set

Attach an arbitrary property to the vertex.

Locations and mapping:

- When you call `set(key, value)` for the first time for a given key, the Vertex assigns the next available `@location(N)` to that property (starting after position). Subsequent calls reuse the same location.
- At render time, shader vertex inputs (declared with `@location(N)`) are derived from the Shader and matched to Vertex/Instance properties by:
  1) explicit location (instance first, then vertex), then
  2) name (instance first, then vertex).
- There is no special-case for location(0) in the mapping; position is just another vertex attribute exposed as `position` with a 2- or 3-component format depending on how you constructed the Vertex.

Planned explicit control:

- You will be able to pin a property to a specific location using a fluent API: `vertex.set(key, value).at(index)`.
- Vertex construction may also support `Vertex::from_shader(&Shader)` to derive an initial layout directly from the shader AST.

## Example

```rust
use fragmentcolor::mesh::{Vertex, VertexValue};
let v = Vertex::new([0.0, 0.0, 0.0]).set("weight", 1.0).set("color",[1.0, 0.0, 0.0]);
```
