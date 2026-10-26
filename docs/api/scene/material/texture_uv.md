# Material::texture_uv

Pick the vertex UV set one texture slot samples: `0` reads `Vertex::UV0`
and `1` reads `Vertex::UV1`. glTF calls this choice `texCoord`. Baked
lighting and occlusion often live on a second UV layout, apart from the
tiled UVs of the color maps. Every slot starts on set 0.

A set other than 0 or 1 returns `MaterialError::UvSet` and keeps the
slot's set. JavaScript throws an `Error`, Python raises `MaterialError`, a
`FragmentColorError` that is also a `ValueError`, Swift throws
`FragmentColorError`, and Kotlin throws `FragmentColorException`. The set
is a count like any other: JavaScript throws for a fraction or a negative
number, and Python takes an `int`, raising `TypeError` for a float and
`InvalidInputError` for an int below 0 or past `2**32 - 1`.

The PBR shader reads `UV1` from every vertex, and a mesh whose vertices
lack it fails to attach with an error that names `uv1`. `Vertex::pbr`
seeds `UV1` at `[0, 0]`, so a slot on set 1 reads that one point until
the mesh sets its own. `Scene::load` gives a primitive without
`TEXCOORD_1` its `TEXCOORD_0` values as `UV1`, so set 1 reads the same UVs
as set 0 there.

The slot's
[`texture_transform`](https://fragmentcolor.org/api/scene/material#texture_transform)
applies to whichever set it reads.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, TextureSlot, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0])
        .set(Vertex::UV0, [0.5, 1.0])
        .set(Vertex::UV1, [0.25, 0.75]),
);

// The occlusion map follows the second UV layout.
let material = Material::pbr().texture_uv(TextureSlot::Occlusion, 1)?;
# let _ = (mesh, material);
# Ok(())
# }
```
