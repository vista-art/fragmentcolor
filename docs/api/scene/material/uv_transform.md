# Material::uv_transform

Set a global UV transform applied to every map sample. Matches glTF's
[`KHR_texture_transform`](https://github.com/KhronosGroup/glTF/tree/main/extensions/2.0/Khronos/KHR_texture_transform)
composition: scale → rotate → offset, with the result fed to every
`textureSample` in the PBR fragment shader.

- `offset` translates the UV after scale + rotation (`[0, 0]` is no translation)
- `scale` multiplies the UV first (`[1, 1]` is no scale)
- `rotation` rotates the scaled UV in radians (positive = counter-clockwise
  in WGSL's `(u, v)` convention)

Defaults to identity (scale `[1, 1]`, rotation `0`, offset `[0, 0]`) so an
unset Material samples textures untransformed. Today the transform is
shared across all five texture slots — per-map transforms (and the per-map
`texCoord` selector that picks `UV0` vs `UV1`) are a follow-up. For glTFs
that carry `KHR_texture_transform` only on the base-color map (the most
common case), the global path is lossless.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

// Tile the texture 4× in both directions, rotate 45°, shift by half a tile.
let brick = Material::pbr()?
    .uv_transform([0.5, 0.0], [4.0, 4.0], std::f32::consts::FRAC_PI_4);
# let _ = brick;
# Ok(())
# }
```
