# Vertex::pbr

Construct a `Vertex` that matches the PBR shader's vertex input layout.
Seeds every attribute [`Material::pbr`](https://fragmentcolor.org/api/scene/material/pbr)
reads (`NORMAL`, `UV0`, `COLOR0`, `UV1`, `TANGENT`) with a neutral identity
default; chain `.set(...)` to override the slots a real mesh has data for.

A vertex built with `Vertex::pbr(pos)` alone renders the same way
[`Scene::load`](https://fragmentcolor.org/api/scene/scene/load) renders a
glTF primitive that carries only POSITION. Face-normal computation, UV
defaults, vertex tint, and tangents all collapse to the values the loader
falls back to. Useful for building PBR meshes by hand without spelling
out every default attribute.

Defaults:

| attribute | default value           | what it means                                   |
| --------- | ----------------------- | ----------------------------------------------- |
| `NORMAL`  | `[0, 0, 1]`             | forward-facing surface                          |
| `UV0`     | `[0, 0]`                | corner of the texture                           |
| `COLOR0`  | `[1, 1, 1, 1]`          | identity vertex tint                            |
| `UV1`     | `[0, 0]`                | unused; only matters for maps that opt into UV1 |
| `TANGENT` | `[1, 0, 0, 1]`          | T = +X, bitangent sign +1                       |

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mesh, Vertex};

let mesh = Mesh::new();
for (pos, uv) in [
    ([0.0, 0.5, 0.0], [0.5, 1.0]),
    ([-0.5, -0.5, 0.0], [0.0, 0.0]),
    ([0.5, -0.5, 0.0], [1.0, 0.0]),
] {
    // Override only what the mesh actually carries; NORMAL / COLOR0 / UV1 /
    // TANGENT use their identity defaults.
    mesh.add_vertex(Vertex::pbr(pos).set(Vertex::UV0, uv));
}
# Ok(())
# }
```
