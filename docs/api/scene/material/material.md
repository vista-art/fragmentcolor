# Material

A `Material` bundles a `Shader` with the glTF 2.0 PBR-MR field set — base
color, metallic, roughness, emissive, normal scale, occlusion strength,
alpha cutoff, plus the matching set of optional textures — and exposes
each as a builder-style setter.

The default `Material::pbr()` constructs the bundle from FragmentColor's
built-in physically-based shader (Cook-Torrance specular with GGX + Smith +
Schlick, Lambertian diffuse with energy conservation), pre-populated with
sensible defaults. You can also call `Material::custom(shader)` to wrap your
own shader; the same setters apply best-effort, no-op-ing for uniform paths
the shader doesn't declare.

Materials are typically combined with a [Mesh](https://fragmentcolor.org/api/geometry/mesh)
into a [Model](https://fragmentcolor.org/api/scene/model) — that's the
object you actually add to a `Pass`. The Material itself doesn't render
anything on its own.

## What lives where

- **Factor uniforms** (base color, metallic, …) are stored on the shader as
  `material.<name>` fields and read every frame.
- **Texture bindings** (base color texture, …) are stored under the
  canonical glTF binding names (`base_color_map`, `metallic_roughness_map`,
  `normal_map`, `occlusion_map`, `emissive_map`).
- **`mesh.model`** is reserved for the per-Model transform — `Model` writes
  it on its own shader copy, not directly on the source Material.

## What's deferred

The factors-only built-in shader does not yet sample the texture bindings.
Setting `material.base_color_texture(...)` stores the texture on a shader
that declares the binding (true for `Material::custom(...)`), and becomes a
no-op-with-debug-log on the default PBR shader. Texture sampling in the
default shader lands in a follow-up; the binding names are stable so you
don't need to rewrite consumer code when it ships.

`alpha_mode` and `double_sided` are not in this MVP — they need
pipeline-state plumbing that's coming next.

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

let material = Material::pbr()
    .base_color([0.85, 0.2, 0.2, 1.0])
    .metallic(0.0)
    .roughness(0.4)
    .emissive([0.0, 0.0, 0.05]);

let model = Model::new(mesh, material);
# let _ = model;
# Ok(())
# }
```
