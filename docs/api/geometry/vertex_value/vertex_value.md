# VertexValue

A vertex or instance attribute value of an exact type. Plain numbers and
number lists are floats (`f32`) in every language, so use a `VertexValue`
when a shader input is an integer, such as `@location(1) id: u32`.

There is one variant per type and size: `F32`, `U32` and `I32` hold one
number, and `F32x2` to `F32x4`, `U32x2` to `U32x4` and `I32x2` to `I32x4` hold
two to four. Rust names the variants in upper case (`VertexValue::U32(7)`),
and JS, Python, Swift and Kotlin build them with lower-case factories
(`VertexValue.u32(7)`, or `VertexValue.u32(7u)` in Kotlin). An integer
that doesn't fit its type is an error.

Pass a `VertexValue` to
[`Vertex::set`](https://fragmentcolor.org/api/geometry/vertex#vertexset) or
[`Instance::set`](https://fragmentcolor.org/api/geometry/instance#instanceset).

## Example

```rust
use fragmentcolor::mesh::{Vertex, VertexValue};

let vertex = Vertex::new([0.0, 0.0]).set("id", VertexValue::U32(7));
# _ = vertex;
```
