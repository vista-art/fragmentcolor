# Material::texture_transform

Give one texture slot its own UV transform, the one glTF's
[`KHR_texture_transform`](https://github.com/KhronosGroup/glTF/tree/main/extensions/2.0/Khronos/KHR_texture_transform)
describes. The slot's UV is scaled first, then rotated, then offset:

- `scale` multiplies the UV (`[1, 1]` keeps it)
- `rotation` turns the scaled UV around the origin, in radians. A positive
  angle turns the UVs counter-clockwise in texture space, where `v` points
  down, so the image itself turns clockwise
- `offset` moves the result, in fractions of the texture (`[0, 0]` keeps it)

Every slot starts with the identity transform. The transform applies to
the UV set that
[`texture_uv`](https://fragmentcolor.org/api/scene/material#texture_uv)
picks for the slot.
[`uv_transform`](https://fragmentcolor.org/api/scene/material#uv_transform)
sets one transform on all five slots, and a `texture_transform` after it
replaces the transform of its slot alone.

Whether a UV past 0 to 1 repeats the texture depends on the sampler. The
PBR shader samples all five slots with one sampler, which takes the
options of the first texture the material sets, in `TextureSlot` order:
base color, metallic-roughness, normal, occlusion, then emissive. A slot
left at its 1×1 default doesn't count. So a material with only an
occlusion map tiles it with the occlusion map's own options, and one with
a clamped base color texture clamps a tiled normal map too.

A number that is NaN or an infinity returns `MaterialError::NotFinite`,
which names the part, such as `Material::texture_transform: rotation must
be finite, got NaN`, and leaves the slot as it was: the setter checks the
offset, the scale and the rotation before it writes any of them. The other
languages also refuse a number past the 32-bit float range, such as
`1e39`, with a message like `Material.textureTransform: rotation must be a
finite number that fits a 32-bit float, got 1e39`. JavaScript throws an
`Error` named `InvalidInputError`, Python raises `InvalidInputError`,
Swift throws `FragmentColorError` and Kotlin throws
`FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, TextureSlot};

// Every map spans the surface twice, and the normal map eight times.
let tiles = Material::pbr()
    .uv_transform([0.0, 0.0], [2.0, 2.0], 0.0)?
    .texture_transform(TextureSlot::Normal, [0.0, 0.0], [8.0, 8.0], 0.0)?;
# let _ = tiles;
# Ok(())
# }
```
