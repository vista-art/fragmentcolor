# PbrMaterial

glTF 2.0's metallic-roughness material, drawn with FragmentColor's built-in
physically-based shader: Cook-Torrance specular with GGX, Smith and Schlick,
and Lambertian diffuse with energy conservation, lit by every
[Light](https://fragmentcolor.org/api/scene/light) that reaches it.

A `PbrMaterial` has glTF's number settings, each a builder-style setter:
`base_color`, `metallic`, `roughness`, `normal_scale`, `occlusion_strength`,
`emissive` and `alpha_cutoff`. It has five maps, each a
[`MaterialMap`](https://fragmentcolor.org/api/scene/materialmap) that sets its
texture, UV set and transform: `base_color_map()`,
`metallic_roughness_map()`, `normal_map()`, `occlusion_map()` and
`emissive_map()`. `alpha_mode` and `double_sided` set how it blends and
which faces it draws.

[`PbrMaterial::new`](https://fragmentcolor.org/api/scene/pbrmaterial#new)
starts from glTF's defaults, as a white surface. Its maps start unbound, and
the renderer resolves each one to a 1×1 default the first time a Pass draws
the material, so the shader has every binding it needs without any further
setup. `PbrMaterial::new` can't fail: the shader parts it composes are
compiled into the library.

A material draws on a [Mesh](https://fragmentcolor.org/api/geometry/mesh)
through a [Model](https://fragmentcolor.org/api/scene/model), the object you
add to a `Pass`. `Model::new` takes a `PbrMaterial` as it takes any
[Material](https://fragmentcolor.org/api/scene/material). The material
draws nothing on its own.

## Cloning is an Arc-share, not a deep copy

`PbrMaterial` clones, and every setter's return value, point at the same
inner shader. Mutating one handle changes the state every other handle
reads:

```text
let red = PbrMaterial::new().base_color([1.0, 0.0, 0.0, 1.0])?;
let same = red.clone();
same.base_color([0.0, 1.0, 0.0, 1.0])?; // both `red` and `same` now read green
```

The same goes for the chained setter's return: `PbrMaterial::new().metallic(...)`
gives you back a handle to the same shader, not a fresh copy. This is what
makes setters chain cheaply and lets one material drive hundreds of Models
without a shader for each.

When you need an independent material, build a fresh `PbrMaterial::new()`
and set the factors and maps you care about.

## What lives where

- **Factor uniforms** (base color, metallic, …) are stored on the shader as
  `material.<name>` fields and read every frame. They belong to the
  material, so the Models that share a material share these values.
- **Maps** are stored under the canonical glTF binding names
  (`base_color_map`, `metallic_roughness_map`, `normal_map`,
  `occlusion_map`, `emissive_map`) and sampled by the shader. Unset maps
  resolve to the renderer's 1×1 defaults, so the shader's bind group is
  always complete.
- **UV sources**: each map samples its own UV set (`UV0` or `UV1`) through
  its own `KHR_texture_transform`. The shader keeps them in `material.uv`,
  five entries in the order of the maps above, each with `uv_set`, `offset`,
  `scale` and `rotation`. `MaterialMap::uv`, `MaterialMap::transform` and
  `PbrMaterial::uv_transform` set them.
- **Per-Model transform** rides the **per-instance vertex attribute** stream
  as `model_0` to `model_3` (four `vec4<f32>` columns of a `mat4x4<f32>`, at
  locations 3 to 6). The renderer fills it from each Model's transform at
  draw time. It's *not* a material uniform, so Models that share one
  material each draw with their own transform.

## Pipeline state

`alpha_mode` (`Opaque` / `Mask` / `Blend`) and `double_sided` are wired
through to the pipeline. See `PbrMaterial::alpha_mode` and
`PbrMaterial::double_sided`. Mask mode uses `material.alpha_cutoff` to discard
transparent fragments. Blend uses standard `SrcAlpha/OneMinusSrcAlpha`
over-blend with depth-write turned off.

Tangent-space normal mapping is built in: the sampled normal is decoded
from the stored `[0, 1]` bytes, its XY components are scaled by
`material.normal_scale`, and the result is rotated from tangent space into
world space using the per-vertex tangent, with the glTF `tangent.w`
handedness flag preserved. For any other shading, bring a
[Shader](https://fragmentcolor.org/api/scene/material#a-shader-as-a-material)
of your own.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mesh, Model, PbrMaterial, Vertex};

let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
]);

let material = PbrMaterial::new()
    .base_color([0.85, 0.2, 0.2, 1.0])?
    .metallic(0.0)?
    .roughness(0.4)?
    .emissive([0.0, 0.0, 0.05])?;

let model = Model::new(mesh, material);
# let _model = model;
# Ok(())
# }
```
