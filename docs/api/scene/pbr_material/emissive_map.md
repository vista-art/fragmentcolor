# PbrMaterial::emissive_map

The emissive map, as a
[`MaterialMap`](https://fragmentcolor.org/api/scene/materialmap) handle that
sets its texture, UV set and transform. The PBR shader samples it in
`fs_main` and multiplies by the factor: per-fragment emission is
`material.emissive * textureSample(emissive_map, pbr_sampler, uv).rgb`, where
`uv` is `slot_uv(material.uv[4], in.uv0, in.uv1)`, the map's own UV set
through its own transform.

A source with no format of its own reads as sRGB in this map, as glTF's
emissive texture does.

Unset, this map draws a 1×1 white default, so the emission falls back to the
`material.emissive` factor as it is.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::PbrMaterial;

// Raw pixels with their size upload when the scene loads.
let glow_bytes: Vec<u8> = vec![
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
];
let material = PbrMaterial::new().emissive([0.8, 0.0, 0.0])?;
let map = material.emissive_map().texture((glow_bytes, [2, 2]))?;
# let _ = map;
# Ok(())
# }
```
