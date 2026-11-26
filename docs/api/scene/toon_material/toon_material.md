# ToonMaterial

A cel-shaded material: each light lights it in flat steps rather than a
smooth gradient, with an optional hard highlight and a rim along the edges
the camera sees. The look reads as drawn rather than photographed, and it
holds up on simple meshes and flat colors.

Each light's diffuse term is cut into
[`bands`](https://fragmentcolor.org/api/scene/toonmaterial#bands) even
steps, three by default, or read from a
[`ramp_map()`](https://fragmentcolor.org/api/scene/toonmaterial#ramp_map)
gradient once one is set. A [`highlight`](https://fragmentcolor.org/api/scene/toonmaterial#highlight)
color adds a hard spot where the light glints, and a
[`rim`](https://fragmentcolor.org/api/scene/toonmaterial#rim) color marks
the faces seen edge-on. Both start black, which turns them off.

`base_color`, `base_color_map()`, `alpha_cutoff`, `alpha_mode` and
`double_sided` mean the same as they do on
[`PbrMaterial`](https://fragmentcolor.org/api/scene/pbrmaterial) and
[`UnlitMaterial`](https://fragmentcolor.org/api/scene/unlitmaterial).

It reads the `camera` and `lights` roots `PbrMaterial` reads, through the
same light loops, so the [Lights](https://fragmentcolor.org/api/scene/light)
on its pass light it as they light a PBR model, at the same brightness, and
the scene's ambient light fills its shadows. It takes the vertex layout
`PbrMaterial` takes, so a
[`Vertex::pbr`](https://fragmentcolor.org/api/geometry/vertex#pbr) mesh, or
a mesh the glTF loader built, draws with any of the three. An outline is
not a material setting: it needs a draw of its own.

A material draws on a [Mesh](https://fragmentcolor.org/api/geometry/mesh)
through a [Model](https://fragmentcolor.org/api/scene/model). `Model::new`
takes a `ToonMaterial` as it takes any
[Material](https://fragmentcolor.org/api/scene/material).

Clones share one shader, as with `PbrMaterial`: a setter on one handle
changes what every handle draws. Build a fresh `ToonMaterial::new()` for an
independent material.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mesh, Model, ToonMaterial, Vertex};

let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]),
]);

let hero = ToonMaterial::new()
    .base_color([0.9, 0.3, 0.2, 1.0])?
    .bands(2)?
    .highlight([1.0, 1.0, 1.0])?
    .rim([0.3, 0.5, 1.0])?;
let model = Model::new(mesh, hero);
# let _model = model;
# Ok(())
# }
```
