# Material

A `Material` bundles a `Shader` with the glTF 2.0 PBR-MR field set — base
color, metallic, roughness, emissive, normal scale, occlusion strength,
alpha cutoff, plus the matching set of optional textures — and exposes
each as a builder-style setter.

The default `Material::pbr` constructs the bundle from FragmentColor's
built-in physically-based shader (Cook-Torrance specular with GGX + Smith +
Schlick, Lambertian diffuse with energy conservation), pre-populated with
sensible defaults. It returns `Result` because the helper snippets it composes
(`mesh/transform`, `material/pbr`) must be available at build time — they
ship in the default `shaders-all` feature, so this is only an error path for
slim opt-out builds. You can also call `Material::custom(shader)` to wrap
your own shader; the same setters apply best-effort, no-op-ing for uniform
paths the shader doesn't declare.

Materials are typically combined with a [Mesh](https://fragmentcolor.org/api/geometry/mesh)
into a [Model](https://fragmentcolor.org/api/scene/model) — that's the
object you actually add to a `Pass`. The Material itself doesn't render
anything on its own.

## What lives where

- **Factor uniforms** (base color, metallic, …) are stored on the shader as
  `material.<name>` fields and read every frame. They're per-Material, so
  many Models that share a Material share these values.
- **Texture bindings** (base color texture, …) are stored under the
  canonical glTF binding names (`base_color_map`, `metallic_roughness_map`,
  `normal_map`, `occlusion_map`, `emissive_map`).
- **Per-Model transform** rides the **per-instance vertex attribute** stream
  at locations 3..6 (four `vec4<f32>` columns of `mat4x4<f32>`), populated by
  `Model::sync_transform`. It's *not* a Material uniform — that way Models
  sharing one Material don't collide on a `mesh.model` slot.

## What's deferred

The factors-only built-in shader does not yet sample the texture bindings.
Setting `material.base_color_texture(...)` stores the texture on a shader
that declares the binding (true for `Material::custom(...)`), and becomes a
no-op-with-debug-log on the default PBR shader. Texture sampling in the
default shader lands in a follow-up; the binding names are stable so you
don't need to rewrite consumer code when it ships.

`alpha_mode` (`Opaque` / `Mask` / `Blend`) and `double_sided` are wired through
to the pipeline. See `Material::alpha_mode` and `Material::double_sided`. Mask
mode uses `material.alpha_cutoff` to discard transparent fragments; Blend uses
standard `SrcAlpha/OneMinusSrcAlpha` over-blend with depth-write turned off.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.5, 1.0]),
);

let material = Material::pbr()?
    .base_color([0.85, 0.2, 0.2, 1.0])
    .metallic(0.0)
    .roughness(0.4)
    .emissive([0.0, 0.0, 0.05]);

let model = Model::new(mesh, material);
# let _ = model;
# Ok(())
# }
```
