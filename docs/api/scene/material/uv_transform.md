# Material::uv_transform

Set one UV transform on all five texture slots, the transform glTF's
[`KHR_texture_transform`](https://github.com/KhronosGroup/glTF/tree/main/extensions/2.0/Khronos/KHR_texture_transform)
describes. Each slot's UV is scaled first, then rotated, then offset:

- `scale` multiplies the UV (`[1, 1]` keeps it)
- `rotation` turns the scaled UV around the origin, in radians. A positive
  angle turns the UVs counter-clockwise in texture space, where `v` points
  down, so the image itself turns clockwise
- `offset` moves the result, in fractions of the texture (`[0, 0]` keeps it)

Every slot starts with the identity transform: scale `[1, 1]`, rotation
`0` and offset `[0, 0]`.
[`texture_transform`](https://fragmentcolor.org/api/scene/material#texture_transform)
sets the transform of one slot, so a call after `uv_transform` replaces
that slot's transform and keeps the other four. Each slot applies its
transform to the UV set that
[`texture_uv`](https://fragmentcolor.org/api/scene/material#texture_uv)
picks for it.

A number that is NaN or an infinity returns `MaterialError::NotFinite`,
which names the part, such as `Material::uv_transform: rotation must be
finite, got NaN`, and leaves every slot as it was: the setter checks the
offset, the scale and the rotation before it writes any of them. The other
languages also refuse a number past the 32-bit float range, such as
`1e39`, with a message like `Material.uvTransform: rotation must be a
finite number that fits a 32-bit float, got 1e39`. JavaScript throws an
`Error` named `InvalidInputError`, Python raises `InvalidInputError`,
Swift throws `FragmentColorError` and Kotlin throws
`FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

// Tile the texture 4× in both directions, rotate 45°, shift by half a tile.
let brick = Material::pbr()
    .uv_transform([0.5, 0.0], [4.0, 4.0], 0.785)?; // 45° in radians
# let _ = brick;
# Ok(())
# }
```
