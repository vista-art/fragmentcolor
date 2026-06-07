# Vertex

A single vertex: a required 2D or 3D position plus any per-vertex attributes your shader declares. Commonly that's `uv` and `color`, but any key the shader's vertex stage reads is fair game. Build vertices and feed them into a [Mesh](https://fragmentcolor.org/api/geometry/mesh) to draw.

## Canonical attribute names

For the channels you'll reach for most often, use the typed constants instead of bare strings. The loader, the shader, and any later glTF import all agree on names without bikeshedding:

Position is set via `Vertex::new(...)` and is not part of the constants table; the rest live as `&'static str` literals on `Vertex`:

| Constant | Value | Notes |
|---|---|---|
| `Vertex::NORMAL` | `"normal"` | Per-vertex normal. |
| `Vertex::TANGENT` | `"tangent"` | Per-vertex tangent (for normal-mapped shading). |
| `Vertex::UV0` / `UV1` | `"uv0"` / `"uv1"` | Texture coordinates. Use the numbered form when a mesh carries multiple UV sets (typical for glTF). |
| `Vertex::COLOR0` / `COLOR1` | `"color0"` / `"color1"` | Vertex colours. |

The constants are plain `&'static str` literals: `vertex.set(Vertex::UV0, [...])` and `vertex.set("uv0", [...])` do the same thing. Mix the two styles freely; the constants exist to prevent typos and to give the glTF loader a stable vocabulary.

## Example

```rust
use fragmentcolor::mesh::Vertex;
let v = Vertex::new([0.0, 0.0, 0.0])
    .set(Vertex::UV0, [0.5, 0.5])
    .set(Vertex::NORMAL, [0.0, 1.0, 0.0]);
```
