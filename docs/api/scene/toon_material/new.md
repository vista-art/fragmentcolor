# ToonMaterial::new

Construct a white, opaque, single-sided `ToonMaterial` that lights in three
bands, with no highlight and no rim. Its two maps start unbound: the
renderer binds a 1×1 white default to each, so the material draws
`base_color` in bands until you set a map through
[`base_color_map()`](https://fragmentcolor.org/api/scene/toonmaterial#base_color_map)
or [`ramp_map()`](https://fragmentcolor.org/api/scene/toonmaterial#ramp_map).
`ToonMaterial::new` can't fail: the shader parts it composes are compiled
into the library.

The shader adds up each light the fragment receives, the scene's ambient
light times the albedo, and the rim, and encodes the result as sRGB on the
`Rgba8Unorm` and `Bgra8Unorm` targets FragmentColor creates, as
`PbrMaterial` does. It reads these vertex inputs from the mesh:

- `@location(0) position : vec3<f32>`, from `Vertex::new([...])` or `Vertex::pbr([...])`.
- `@location(1) normal   : vec3<f32>`, set as `.set(Vertex::NORMAL, ...)`.
- `@location(2) uv0      : vec2<f32>`, set as `.set(Vertex::UV0, ...)`.
- `@location(7) color0   : vec4<f32>`, set as `.set(Vertex::COLOR0, ...)`, which tints the base color.
- `@location(8) uv1      : vec2<f32>`, set as `.set(Vertex::UV1, ...)`, which the base color map samples on UV set 1.

`Vertex::pbr` sets each of them to a neutral value. Locations 3 to 6 hold
`model_0` to `model_3`, the columns of the Model's transform.

## Defaults (post-construction)

| key                       | value          |
| ------------------------- | -------------- |
| `material.base_color`     | `[1, 1, 1, 1]` |
| `material.bands`          | `3`            |
| `material.highlight`      | `[0, 0, 0]`    |
| `material.highlight_size` | `0.05`         |
| `material.rim`            | `[0, 0, 0]`    |
| `material.rim_width`      | `0.2`          |
| `material.alpha_cutoff`   | `0.5`          |
| `base_color_map`          | 1×1 white      |
| `ramp_map`                | 1×1, no ramp   |
| `camera.view_proj`        | identity 4×4   |
| `camera.position`         | `[0, 0, 0]`    |
| `lights.ambient`          | `[0.03; 3]`    |
| placeholder light         | white, directional, pointing down `[0, -1, 0]` |

The placeholder light lights a fresh material until the first
[Light](https://fragmentcolor.org/api/scene/light) reaches its shader,
and returns once no Light does, as on `PbrMaterial`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::ToonMaterial;

let grass = ToonMaterial::new().base_color([0.3, 0.7, 0.2, 1.0])?;
# let _grass = grass;
# Ok(())
# }
```
