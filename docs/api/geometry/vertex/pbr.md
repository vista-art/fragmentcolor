# Vertex::pbr

Construct a `Vertex` that matches the PBR shader's vertex input layout.
Seeds every attribute [`Material::pbr`](https://fragmentcolor.org/api/scene/material#pbr)
reads (`NORMAL`, `UV0`, `COLOR0`, `UV1`, `TANGENT`) with a neutral identity
default; chain `.set(...)` to override the slots a real mesh has data for.

The defaults are the values
[`Scene::load`](https://fragmentcolor.org/api/scene/scene#load) falls back
to for a glTF primitive that lacks those attributes, with two exceptions.
The loader computes a missing normal from the faces, where `Vertex::pbr`
points every normal along +Z. And the loader copies a primitive's
`TEXCOORD_0` into `UV1` when it has no `TEXCOORD_1`, where `Vertex::pbr`
keeps `UV1` at `[0, 0]` after you set `UV0`, until you set `UV1` too.
Useful for building PBR meshes by hand without spelling out every default
attribute.

Defaults:

| attribute | default value           | what it means                                   |
| --------- | ----------------------- | ----------------------------------------------- |
| `NORMAL`  | `[0, 0, 1]`             | forward-facing surface                          |
| `UV0`     | `[0, 0]`                | corner of the texture                           |
| `COLOR0`  | `[1, 1, 1, 1]`          | identity vertex tint                            |
| `UV1`     | `[0, 0]`                | read by slots `texture_uv` puts on UV set 1     |
| `TANGENT` | `[1, 0, 0, 1]`          | T = +X, bitangent sign +1                       |

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mesh, Vertex};

// Build a triangle and set only what the mesh carries. NORMAL, COLOR0,
// UV1 and TANGENT keep their identity defaults from Vertex::pbr.
let mesh = Mesh::new();
mesh.add_vertex(Vertex::pbr([ 0.0,  0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]));
mesh.add_vertex(Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]));
mesh.add_vertex(Vertex::pbr([ 0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]));
# Ok(())
# }
```
