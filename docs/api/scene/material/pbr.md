# Material::pbr

Construct a `Material` whose shader is FragmentColor's built-in
physically-based-rendering default. The bundle ships pre-configured with
glTF 2.0 PBR-MR defaults, lightly tweaked so a freshly-constructed Material
renders as a clean white surface under the default light rather than dark
gunmetal.

The shader uses:

- **Cook-Torrance specular** with GGX normal distribution, Smith geometry,
  and Schlick Fresnel.
- **Lambertian diffuse** with energy conservation against the specular
  Fresnel and metalness.
- **One directional light** (uniform `light`), one camera (uniform `camera`),
  per-Model transform via `mesh.model`.

The vertex inputs the shader expects, in this order:

- `@location(0) position : vec3<f32>` — set as `Vertex::new([...])`.
- `@location(1) normal   : vec3<f32>` — set as `.set(Vertex::NORMAL, ...)`.
- `@location(2) uv0      : vec2<f32>` — set as `.set(Vertex::UV0, ...)`.

If your Mesh's first vertex doesn't carry these three properties in this
order, attaching it to the Material's shader will fail with a layout
mismatch — use `Material::custom(...)` to bring your own vertex layout.

## Defaults (post-construction)

| key                          | value                |
| ---------------------------- | -------------------- |
| `material.base_color`        | `[1, 1, 1, 1]`       |
| `material.metallic`          | `0.0`                |
| `material.roughness`         | `1.0`                |
| `material.normal_scale`      | `1.0`                |
| `material.occlusion_strength`| `1.0`                |
| `material.emissive`          | `[0, 0, 0]`          |
| `material.alpha_cutoff`      | `0.5`                |
| `mesh.model`                 | identity 4×4         |
| `camera.view_proj`           | identity 4×4         |
| `camera.position`            | `[0, 0, 0]`          |
| `light.direction`            | `[0, -1, 0]` (down)  |
| `light.color`                | `[1, 1, 1]`          |

You set `camera.view_proj` and `camera.position` directly via
`material.shader().set(...)` — the camera is the user's domain, not the
Material's. A Camera object is planned for a follow-up.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let bronze = Material::pbr()?
    .base_color([0.8, 0.5, 0.2, 1.0])
    .metallic(1.0)
    .roughness(0.3);

# let _ = bronze;
# Ok(())
# }
```
