# ToonMaterial::ramp_map

The gradient that lights the material in place of its
[`bands`](https://fragmentcolor.org/api/scene/toonmaterial#bands), as a
[`MaterialMap`](https://fragmentcolor.org/api/scene/materialmap) handle that
sets its texture. Each light reads it left to right by the cosine `ndl`
between the normal and the light, at `ndl * 0.5 + 0.5`: the left half
colors the faces turned away from the light, the middle the edge of the
shadow, and the right edge the faces turned straight at it. The ramp's color
times the base color is that light's diffuse term, so a colored ramp tints
the shadows.

The ramp takes over once it holds a texture wider than one texel. The ramp
samples its middle row, so a ramp one texel tall is enough. Its texture's
own sampler applies: set `smooth: false` for hard steps between its texels.
A source with no format of its own reads as sRGB, as a painted gradient is.

The ramp reads the light, not the mesh's UVs, so its
[`uv`](https://fragmentcolor.org/api/scene/materialmap#uv) and
[`transform`](https://fragmentcolor.org/api/scene/materialmap#transform)
return `MaterialError::NoUv`, which names the call, and change nothing.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::ToonMaterial;

// Deep blue shadows, a warm middle tone and full light, one texel each.
let ramp: Vec<u8> = vec![
    20, 30, 90, 255, 20, 30, 90, 255,
    200, 140, 110, 255, 255, 255, 255, 255,
];
let material = ToonMaterial::new();
let map = material.ramp_map().texture((ramp, [4, 1]))?;
# let _ = map;
# Ok(())
# }
```
