# PbrMaterial::normal_map

The tangent-space normal map, as a
[`MaterialMap`](https://fragmentcolor.org/api/scene/materialmap) handle that
sets its texture, UV set and transform. The PBR shader samples the map in
`fs_main`, decodes the stored `[0, 1]` bytes into `[-1, 1]` floats, scales the
XY components by `material.normal_scale`, and rotates the perturbed normal
from tangent space into world space using the per-vertex tangent (the glTF
`tangent.w` handedness flag is preserved).

A source with no format of its own reads as linear in this map, since it
holds directions rather than color.

Unset, this map draws a 1×1 flat tangent-space default `(128, 128, 255, 255)`,
so the decoded normal is `(0, 0, 1)` and the world normal passes through
unchanged.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::PbrMaterial;

// Raw pixels with their size upload when the scene loads.
let normal_map_bytes: Vec<u8> = vec![
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
];
let material = PbrMaterial::new().normal_scale(1.2)?;
let map = material.normal_map().texture((normal_map_bytes, [2, 2]))?;
# let _ = map;
# Ok(())
# }
```
