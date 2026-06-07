# Material::pbr

Construct a `Material` whose shader is FragmentColor's built-in
physically-based-rendering default. The bundle ships pre-configured with
glTF 2.0 PBR-MR defaults, lightly tweaked so a freshly-constructed Material
renders as a clean white surface under the default light rather than dark
gunmetal.

The five glTF texture slots (`base_color_map`, `metallic_roughness_map`,
`normal_map`, `occlusion_map`, `emissive_map`) start unbound; the renderer
resolves each one to a sensible 1×1 default from its lazy texture cache the
first time a Pass renders the Material. That way the underlying shader has
every binding it needs and the per-factor render is correct even when no map
is attached. Call the texture setters to override any of them.

The shader uses:

- **Cook-Torrance specular** with GGX normal distribution, Smith geometry,
  and Schlick Fresnel.
- **Lambertian diffuse** with energy conservation against the specular
  Fresnel and metalness.
- **One directional light** (uniform `light`), one camera (uniform `camera`),
  per-Model transform via the instance-attribute stream.
- **glTF 2.0 PBR-MR texture sampling**: five sampled maps (`base_color_map`,
  `metallic_roughness_map`, `normal_map`, `occlusion_map`, `emissive_map`)
  combined with their matching factors per the spec.

The vertex inputs the shader expects, in this order:

- `@location(0) position : vec3<f32>`, set as `Vertex::new([...])`.
- `@location(1) normal   : vec3<f32>`, set as `.set(Vertex::NORMAL, ...)`.
- `@location(2) uv0      : vec2<f32>`, set as `.set(Vertex::UV0, ...)`.

If your Mesh's first vertex doesn't carry these three properties in this
order, attaching it to the Material's shader will fail with a layout
mismatch. Use `Material::custom(...)` to bring your own vertex layout.

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
| `base_color_map`             | 1×1 white            |
| `metallic_roughness_map`     | 1×1 (R=0, G=1, B=1)  |
| `normal_map`                 | 1×1 flat (128,128,255) |
| `occlusion_map`              | 1×1 white            |
| `emissive_map`               | 1×1 white            |
| `camera.view_proj`           | identity 4×4         |
| `camera.position`            | `[0, 0, 0]`          |
| `light.direction`            | `[0, -1, 0]` (down)  |
| `light.color`                | `[1, 1, 1]`          |

The camera and light are the user's domain, not the Material's. Pass them
to [`Pass::add`](https://fragmentcolor.org/api/core/pass#add). The
typed [Camera](https://fragmentcolor.org/api/scene/camera) and
[Light](https://fragmentcolor.org/api/scene/light) handles seed the matching
`camera.*` / `light.*` uniforms and propagate any later updates back into
the Material's shader. Dropping down to `material.shader().set(...)` directly
is still supported if you need it.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let bronze = Material::pbr()
    .base_color([0.8, 0.5, 0.2, 1.0])
    .metallic(1.0)
    .roughness(0.3);

# let _bronze = bronze;
# Ok(())
# }
```
