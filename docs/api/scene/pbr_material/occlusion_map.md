# PbrMaterial::occlusion_map

The ambient-occlusion map, as a
[`MaterialMap`](https://fragmentcolor.org/api/scene/materialmap) handle that
sets its texture, UV set and transform. Following glTF 2.0, only the red
channel is read. The sampled value is blended toward `1.0` by
`1 - material.occlusion_strength` inside the PBR shader, then multiplied into
the diffuse term.

A source with no format of its own reads as linear in this map, since it
holds data rather than color. Baked occlusion often lives on a second UV
layout, which the map's [`uv`](https://fragmentcolor.org/api/scene/materialmap#uv)
picks.

Unset, this map draws a 1×1 white default, so the occlusion factor is `1.0`
and no darkening is applied.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::PbrMaterial;

// Raw pixels with their size upload when the scene loads.
let ao_bytes: Vec<u8> = vec![
    220, 0, 0, 255,
    180, 0, 0, 255,
    200, 0, 0, 255,
    160, 0, 0, 255,
];
let material = PbrMaterial::new();
let map = material.occlusion_map().texture((ao_bytes, [2, 2]))?;
# let _ = map;
# Ok(())
# }
```
