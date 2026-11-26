# UnlitMaterial

A material that draws its base color as it is, with no lighting: glTF's
`KHR_materials_unlit`. Each fragment is `base_color` times its map's sample
times the vertex color, encoded for the target. Reach for it for UI,
sprites, models whose lighting is baked into their textures, and debug
views. The [glTF loader](https://fragmentcolor.org/api/scene/scene#load)
builds one for every material that carries `KHR_materials_unlit`.

An `UnlitMaterial` has two number settings, `base_color` and
`alpha_cutoff`, and one map, `base_color_map()`, a
[`MaterialMap`](https://fragmentcolor.org/api/scene/materialmap) that sets
its texture, UV set and transform. `alpha_mode` and `double_sided` set how
it blends and which faces it draws. These calls mean the same as they do on
[`PbrMaterial`](https://fragmentcolor.org/api/scene/pbrmaterial).

It reads the `camera` root and no lights, so a
[Light](https://fragmentcolor.org/api/scene/light) on its pass costs it
nothing and changes nothing. It takes the vertex layout `PbrMaterial` takes,
so a [`Vertex::pbr`](https://fragmentcolor.org/api/geometry/vertex#pbr) mesh,
or a mesh the glTF loader built, draws with either material.

A material draws on a [Mesh](https://fragmentcolor.org/api/geometry/mesh)
through a [Model](https://fragmentcolor.org/api/scene/model). `Model::new`
takes an `UnlitMaterial` as it takes any
[Material](https://fragmentcolor.org/api/scene/material).

Clones share one shader, as with `PbrMaterial`: a setter on one handle
changes what every handle draws. Build a fresh `UnlitMaterial::new()` for an
independent material.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mesh, Model, UnlitMaterial, Vertex};

let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
]);

let badge = UnlitMaterial::new().base_color([1.0, 0.8, 0.1, 1.0])?;
let model = Model::new(mesh, badge);
# let _model = model;
# Ok(())
# }
```
