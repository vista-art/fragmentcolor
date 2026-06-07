# Material

A `Material` bundles a `Shader` with the glTF 2.0 PBR-MR field set (base
color, metallic, roughness, emissive, normal scale, occlusion strength,
alpha cutoff, plus the matching set of optional textures) and exposes
each as a builder-style setter.

The default `Material::pbr` constructs the bundle from FragmentColor's
built-in physically-based shader (Cook-Torrance specular with GGX + Smith +
Schlick, Lambertian diffuse with energy conservation, glTF-spec texture
sampling on top of the factor uniforms), pre-populated with sensible
defaults. The five glTF texture slots start unbound; the renderer resolves
each one to a 1×1 default from its lazy texture cache the first time a Pass
renders the Material, so the Material's shader has every binding it needs
without any further setup. `Material::pbr` returns `Result` because the
helper snippets it composes (`mesh/transform`, `material/pbr`) must be
available at build time. They ship in the default `shaders-all` feature,
so this is only an error path for slim opt-out builds. You can also call
`Material::custom(shader)` to wrap your own shader; the same setters apply
best-effort, no-op-ing for uniform paths the shader doesn't declare.

Materials are typically combined with a [Mesh](https://fragmentcolor.org/api/geometry/mesh)
into a [Model](https://fragmentcolor.org/api/scene/model), which is the
object you actually add to a `Pass`. The Material itself doesn't render
anything on its own.

## Cloning is an Arc-share, not a deep copy

`Material` clones (and every setter's return value) point at the same
inner shader. Mutating one handle changes the state every other handle
reads:

```text
let red = Material::pbr().base_color([1.0, 0.0, 0.0, 1.0]);
let same = red.clone();
same.base_color([0.0, 1.0, 0.0, 1.0]); // both `red` and `same` now read green
```

Same goes for the chained-setter return: `Material::pbr().metallic(...)`
gives you back a handle to the same shader, not a fresh copy. This is the
mechanic that makes setters chain cheaply and lets one Material drive
hundreds of Models without per-instance shader duplication.

When you genuinely need an independent material, build a fresh
`Material::pbr()` (and re-set the factor / texture slots you care about).
The "many handles to one shader" share semantics is the right default;
the explicit-fresh-Material path is the escape hatch.

## What lives where

- **Factor uniforms** (base color, metallic, …) are stored on the shader as
  `material.<name>` fields and read every frame. They're per-Material, so
  many Models that share a Material share these values.
- **Texture bindings** (base color texture, …) are stored under the
  canonical glTF binding names (`base_color_map`, `metallic_roughness_map`,
  `normal_map`, `occlusion_map`, `emissive_map`) and sampled by the default
  shader. Unset slots resolve to the renderer's 1×1 defaults so the shader's
  bind group is always complete.
- **Per-Model transform** rides the **per-instance vertex attribute** stream
  at locations 3..6 (four `vec4<f32>` columns of `mat4x4<f32>`), populated by
  `Model::sync_transform`. It's *not* a Material uniform, so Models
  sharing one Material don't collide on a `mesh.model` slot.

## Pipeline state

`alpha_mode` (`Opaque` / `Mask` / `Blend`) and `double_sided` are wired
through to the pipeline. See `Material::alpha_mode` and
`Material::double_sided`. Mask mode uses `material.alpha_cutoff` to discard
transparent fragments; Blend uses standard `SrcAlpha/OneMinusSrcAlpha`
over-blend with depth-write turned off.

Tangent-space TBN normal mapping is supported via the default PBR shader:
the sampled normal is decoded from the stored `[0, 1]` bytes, the XY
components are scaled by `material.normal_scale`, and the result is rotated
from tangent space into world space using the per-vertex tangent (with the
glTF `tangent.w` handedness flag preserved). For custom TBN beyond the
bundled implementation, use `Material::custom`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);

let material = Material::pbr()
    .base_color([0.85, 0.2, 0.2, 1.0])
    .metallic(0.0)
    .roughness(0.4)
    .emissive([0.0, 0.0, 0.05]);

let model = Model::new(mesh, material);
# let _model = model;
# Ok(())
# }
```
