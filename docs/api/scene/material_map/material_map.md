# MaterialMap

One texture map of a material: the texture it samples, the vertex UV set it
reads, and the UV transform it applies, the way glTF's `textureInfo` and
[`KHR_texture_transform`](https://github.com/KhronosGroup/glTF/tree/main/extensions/2.0/Khronos/KHR_texture_transform)
describe a map.

A material hands out one handle per map, such as
[`PbrMaterial::base_color_map`](https://fragmentcolor.org/api/scene/pbrmaterial#base_color_map)
or [`PbrMaterial::normal_map`](https://fragmentcolor.org/api/scene/pbrmaterial#normal_map).
You don't build a `MaterialMap` yourself. Each setter writes into the
material the map came from and returns the map, so the calls chain:

- [`texture`](https://fragmentcolor.org/api/scene/materialmap#texture) sets
  the texture, from a `Texture` or from any input `Renderer::create_texture`
  takes.
- [`uv`](https://fragmentcolor.org/api/scene/materialmap#uv) picks the UV set,
  `0` or `1`.
- [`transform`](https://fragmentcolor.org/api/scene/materialmap#transform)
  scales, rotates and offsets the UVs.

A map clones as a handle to the same material, as the material does, so a
setter through any copy changes what every Model that draws with the
material shows.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::PbrMaterial;

let flat: Vec<u8> = vec![128, 128, 255, 255];
let gold = PbrMaterial::new().metallic(1.0)?.roughness(0.3)?;

// A normal map tiled four times over the second UV layout.
let map = gold
    .normal_map()
    .texture((flat, [1, 1]))?
    .uv(1)?
    .transform([0.0, 0.0], [4.0, 4.0], 0.0)?;
# let _ = map;
# Ok(())
# }
```
