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
- **Clustered lights**: as many lights as the device allows, in the
  read-only storage roots `lights` and `light_grid` that the renderer fills.
  Each fragment reads only the lights whose range reaches its cluster, plus
  every light without a range. See [Light](https://fragmentcolor.org/api/scene/light)
  for the cost of each kind and the cap on each device, and
  [`Material::custom`](https://fragmentcolor.org/api/scene/material#custom)
  for the layout of the roots.
- **One camera** (uniform `camera`), and the per-Model transform via the
  instance-attribute stream.
- **glTF 2.0 PBR-MR texture sampling**: five sampled maps (`base_color_map`,
  `metallic_roughness_map`, `normal_map`, `occlusion_map`, `emissive_map`)
  combined with their matching factors per the spec.
- **Output for the target**: lighting is computed in linear light. On the
  `Rgba8Unorm` and `Bgra8Unorm` targets FragmentColor creates, which hold
  colors encoded for display, the shader encodes it as sRGB. An sRGB target
  encodes on write and a float target keeps linear light for the passes after
  it, so both get linear light. The renderer picks per pipeline from the
  target's format, through the shader's `override ENCODE_SRGB: bool`. A custom
  shader that declares the same override gets the same value.

The vertex inputs the shader reads from the mesh, each matched by name, so
the order a vertex sets them in doesn't matter:

- `@location(0) position : vec3<f32>`, a 3D position from `Vertex::new([...])` or `Vertex::pbr([...])`.
- `@location(1) normal   : vec3<f32>`, set as `.set(Vertex::NORMAL, ...)`.
- `@location(2) uv0      : vec2<f32>`, set as `.set(Vertex::UV0, ...)`.
- `@location(7) color0   : vec4<f32>`, set as `.set(Vertex::COLOR0, ...)`, which tints the base color.
- `@location(8) uv1      : vec2<f32>`, set as `.set(Vertex::UV1, ...)`, which the texture slots on UV set 1 sample.
- `@location(9) tangent  : vec4<f32>`, set as `.set(Vertex::TANGENT, ...)`, with the bitangent sign in `w`.

Locations 3 to 6 hold `model_0` to `model_3`, the columns of the Model's
transform, which come from the Model rather than the mesh. The shader has
no default for any mesh input, so a Model whose mesh lacks one fails to
attach with an error that names the input. `Vertex::pbr` sets every input
after the position to a neutral value. Use `Material::custom(...)` to bring
your own vertex layout.

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
| `lights.ambient`             | `[0.03, 0.03, 0.03]` |
| placeholder light            | white, directional, pointing down `[0, -1, 0]` |

The placeholder light lights a fresh Material until the first
[Light](https://fragmentcolor.org/api/scene/light) reaches its shader,
and returns once no Light does.

The camera and lights are the user's domain, not the Material's. Pass them
to [`Pass::add`](https://fragmentcolor.org/api/core/pass#add). The typed
[Camera](https://fragmentcolor.org/api/scene/camera) handle seeds the
`camera.*` uniforms and propagates later updates into the Material's shader,
and the renderer packs every Light that reaches the shader at each render.
`material.shader().set(...)` still sets `camera.*` and `lights.ambient`
directly. The lights themselves have no keys: `set` and `get` on
`lights.count` or `lights.lights[i]` return `ShaderError::UniformNotFound`.

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
