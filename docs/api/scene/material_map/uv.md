# MaterialMap::uv

Pick the vertex UV set the map samples, and return the map: `0` reads `Vertex::UV0`
and `1` reads `Vertex::UV1`. glTF calls this choice `texCoord`. Baked
lighting and occlusion often live on a second UV layout, apart from the
tiled UVs of the color maps. Every map starts on set 0.

A set other than 0 or 1 returns `MaterialError::UvSet`, such as
`PbrMaterial::occlusion_map().uv takes UV set 0 or 1, got 2`, and keeps the
map's set. JavaScript throws an `Error` named `MaterialError`, Python raises `MaterialError`, a
`FragmentColorError` that is also a `ValueError`, Swift throws
`FragmentColorError`, and Kotlin throws `FragmentColorException`. The set
is a count like any other: JavaScript throws for a fraction or a negative
number, and Python takes an `int`, raising `TypeError` for a float and
`InvalidInputError` for an int below 0 or past `2**32 - 1`.

The PBR shader reads `UV1` from every vertex, and a mesh whose vertices
lack it fails to attach with an error that names `uv1`. `Vertex::pbr`
seeds `UV1` at `[0, 0]`, so a map on set 1 reads that one point until
the mesh sets its own. `Scene::load` gives a primitive without
`TEXCOORD_1` its `TEXCOORD_0` values as `UV1`, so set 1 reads the same UVs
as set 0 there.

The map's
[`transform`](https://fragmentcolor.org/api/scene/materialmap#transform)
applies to whichever set it reads.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mesh, PbrMaterial, Vertex};

let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0])
        .set(Vertex::UV0, [0.5, 1.0])
        .set(Vertex::UV1, [0.25, 0.75]),
    Vertex::pbr([-0.5, -0.5, 0.0])
        .set(Vertex::UV0, [0.0, 0.0])
        .set(Vertex::UV1, [0.0, 0.5]),
    Vertex::pbr([0.5, -0.5, 0.0])
        .set(Vertex::UV0, [1.0, 0.0])
        .set(Vertex::UV1, [0.5, 0.5]),
]);

// The occlusion map follows the second UV layout.
let material = PbrMaterial::new();
let map = material.occlusion_map().uv(1)?;
# let _ = (mesh, map);
# Ok(())
# }
```
