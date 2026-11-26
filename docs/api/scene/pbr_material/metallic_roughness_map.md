# PbrMaterial::metallic_roughness_map

The metallic-roughness map, as a
[`MaterialMap`](https://fragmentcolor.org/api/scene/materialmap) handle that
sets its texture, UV set and transform. Following glTF 2.0, the green channel
encodes per-fragment roughness and the blue channel encodes metallic. Both are
multiplied by their factors at sample time inside the PBR shader.

A source with no format of its own reads as linear in this map, since it
holds data rather than color.

Unset, this map draws a 1×1 default with `(R=0, G=1, B=1, A=1)`, so the
multiplication passes both `material.metallic` and `material.roughness`
through unchanged.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::PbrMaterial;

// Raw pixels with their size upload when the scene loads.
let mr_map_bytes: Vec<u8> = vec![
    0, 200, 50, 255,
    0, 240, 80, 255,
    0, 180, 30, 255,
    0, 220, 60, 255,
];
let material = PbrMaterial::new();
let map = material.metallic_roughness_map().texture((mr_map_bytes, [2, 2]))?;
# let _ = map;
# Ok(())
# }
```
