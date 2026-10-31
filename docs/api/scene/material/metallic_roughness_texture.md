# Material::metallic_roughness_texture

Bind a texture to the canonical `metallic_roughness_map` slot. Following
glTF 2.0, the green channel encodes per-fragment roughness, the blue channel
encodes metallic. Both are multiplied by their respective factors at sample
time inside the default PBR shader.

Unset, this slot resolves to a 1×1 default with `(R=0, G=1, B=1, A=1)` so
the factor multiplication passes both `material.metallic` and
`material.roughness` through unchanged.

Takes the inputs [`Material::base_color_texture`](https://fragmentcolor.org/api/scene/material#base_color_texture) takes, spelled as each language's `create_texture` spells them: a `Texture`, or encoded bytes, raw pixels with their size, bytes with a format or with every option, a path or a URL, alone or with a format or every option, or a `Mipmap`, alone or with options. The Material keeps a source as a map, and each `Renderer` uploads a copy of its own when it loads the scene. `base_color_texture` tells how a map loads, what it holds in memory and how it fails.

## Example

```rust
use fragmentcolor::Material;

// Raw pixels with their size upload when the scene loads.
let mr_map_bytes: Vec<u8> = vec![
    0, 200, 50, 255,
    0, 240, 80, 255,
    0, 180, 30, 255,
    0, 220, 60, 255,
];
let mat = Material::pbr().metallic_roughness_texture((mr_map_bytes, [2, 2]));
# let _ = mat;
```
