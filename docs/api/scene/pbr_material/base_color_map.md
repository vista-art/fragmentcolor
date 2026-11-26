# PbrMaterial::base_color_map

The base color map, as a
[`MaterialMap`](https://fragmentcolor.org/api/scene/materialmap) handle that
sets its texture, UV set and transform. The PBR shader samples it in
`fs_main` and multiplies by the factor. The albedo for each fragment is
`material.base_color * textureSample(base_color_map, pbr_sampler, uv) * vertex_color`,
where `uv` is `slot_uv(material.uv[0], in.uv0, in.uv1)`: the UV set the map's
[`uv`](https://fragmentcolor.org/api/scene/materialmap#uv) picks, through its
[`transform`](https://fragmentcolor.org/api/scene/materialmap#transform).

A source with no format of its own reads as sRGB in this map, as glTF's base
color does.

Unset, this map draws a 1×1 white default, so `PbrMaterial::new()` without a
texture renders under the factor alone.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::PbrMaterial;

// Raw pixels with their size upload when the scene loads.
let tint: Vec<u8> = vec![255, 128, 0, 255];
let material = PbrMaterial::new();
let map = material.base_color_map().texture((tint, [1, 1]))?;
# let _ = map;
# Ok(())
# }
```
