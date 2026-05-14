# Material

A `Material` bundles a `Shader` with the glTF 2.0 PBR-MR field set — base
color, metallic, roughness, emissive, normal scale, occlusion strength,
alpha cutoff, plus the matching set of optional textures — and exposes
each as a builder-style setter.

The default `Material::pbr` constructs the bundle from FragmentColor's
built-in physically-based shader (Cook-Torrance specular with GGX + Smith +
Schlick, Lambertian diffuse with energy conservation, glTF-spec texture
sampling on top of the factor uniforms), pre-populated with sensible
defaults. It takes a `&Renderer` so the five glTF texture slots come pre-
bound to 1×1 defaults pulled from the renderer's lazy texture cache —
meaning the Material's shader has every binding it needs the moment you
build it. It returns `Result` because the helper snippets it composes
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
  `normal_map`, `occlusion_map`, `emissive_map`) and sampled by the default
  shader. Unset slots resolve to the renderer's 1×1 defaults so the shader's
  bind group is always complete.
- **Per-Model transform** rides the **per-instance vertex attribute** stream
  at locations 3..6 (four `vec4<f32>` columns of `mat4x4<f32>`), populated by
  `Model::sync_transform`. It's *not* a Material uniform — that way Models
  sharing one Material don't collide on a `mesh.model` slot.

## What's deferred

Normal mapping in the default shader is currently a placeholder: the sampled
normal is read and the `normal_scale` factor is applied, but the full
tangent-space-to-world TBN transform is a follow-up — meshes that need
accurate bump shading should either ship pre-baked world-space normals or
use `Material::custom` for now.

`alpha_mode` (`Opaque` / `Mask` / `Blend`) and `double_sided` are wired through
to the pipeline. See `Material::alpha_mode` and `Material::double_sided`. Mask
mode uses `material.alpha_cutoff` to discard transparent fragments; Blend uses
standard `SrcAlpha/OneMinusSrcAlpha` over-blend with depth-write turned off.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Renderer, Vertex};

let renderer = Renderer::new();
let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.5, 1.0])
        .set(Vertex::COLOR0, [1.0, 1.0, 1.0, 1.0])
        .set(Vertex::UV1, [0.0, 0.0]),
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
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
