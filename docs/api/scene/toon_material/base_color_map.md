# ToonMaterial::base_color_map

The material's color map, as a
[`MaterialMap`](https://fragmentcolor.org/api/scene/materialmap) handle that
sets its texture, UV set and transform. Its sample multiplies
`material.base_color` and the vertex color, at the UV set the map's
[`uv`](https://fragmentcolor.org/api/scene/materialmap#uv) picks, through its
[`transform`](https://fragmentcolor.org/api/scene/materialmap#transform).

A source with no format of its own reads as sRGB in this map, as glTF's base
color does. Unset, the map draws a 1×1 white default, so the material draws
its base color alone.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::ToonMaterial;

// Raw pixels with their size upload when the scene loads.
let pixels: Vec<u8> = vec![255, 128, 0, 255];
let material = ToonMaterial::new();
let map = material.base_color_map().texture((pixels, [1, 1]))?;
# let _ = map;
# Ok(())
# }
```
