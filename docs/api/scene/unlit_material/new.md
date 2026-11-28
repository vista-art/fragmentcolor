# UnlitMaterial::new

Construct a white, opaque, single-sided `UnlitMaterial`. Its base color map
starts unbound, and the renderer binds a 1×1 white default in its place, so
the material draws `base_color` alone until you set a map through
[`base_color_map()`](https://fragmentcolor.org/api/scene/unlitmaterial#base_color_map).
`UnlitMaterial::new` can't fail: the shader parts it composes are compiled
into the library.

The shader multiplies the base color, the map's sample and the vertex color,
and encodes the result as sRGB on the `Rgba8Unorm` and `Bgra8Unorm` targets
FragmentColor creates, as `PbrMaterial` does. It reads these vertex inputs
from the mesh:

- `@location(0) position : vec3<f32>`, from `Vertex::new([...])` or `Vertex::pbr([...])`.
- `@location(2) uv0      : vec2<f32>`, set as `.set(Vertex::UV0, ...)`.
- `@location(7) color0   : vec4<f32>`, set as `.set(Vertex::COLOR0, ...)`, which tints the base color.
- `@location(8) uv1      : vec2<f32>`, set as `.set(Vertex::UV1, ...)`, which the map samples on UV set 1.

`Vertex::pbr` sets each of them to a neutral value. Locations 3 to 6 hold
`model_0` to `model_3`, the columns of the Model's transform.

## Defaults (post-construction)

| key                     | value          |
| ----------------------- | -------------- |
| `material.base_color`   | `[1, 1, 1, 1]` |
| `material.alpha_cutoff` | `0.5`          |
| `base_color_map`        | 1×1 white      |
| `camera.view_proj`      | identity 4×4   |
| `camera.position`       | `[0, 0, 0]`    |

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::UnlitMaterial;

let label = UnlitMaterial::new().base_color([0.1, 0.6, 1.0, 1.0])?;
# let _label = label;
# Ok(())
# }
```
