# Material::emissive_texture

Bind an emissive map to the canonical `emissive_map` slot. The default PBR
shader samples it in `fs_main` and multiplies by the factor: per-fragment
emission is `material.emissive * textureSample(emissive_map, pbr_sampler, uv).rgb`,
where `uv` is `slot_uv(material.uv[4], in.uv0, in.uv1)`, the slot's own UV
set through its own transform.

Unset, this slot resolves to a 1×1 white default so the multiplied
emission falls back to the `material.emissive` factor as-is.

Takes the inputs [`Material::base_color_texture`](https://fragmentcolor.org/api/scene/material#base_color_texture) takes, spelled as each language's `create_texture` spells them: a `Texture`, or encoded bytes, raw pixels with their size, bytes with a format or with every option, a path, a URL or a `Mipmap`. The Material keeps a source as a map, and each `Renderer` uploads a copy of its own when it loads the scene. `base_color_texture` tells how a map loads, what it holds in memory and how it fails.

## Example

```rust
use fragmentcolor::Material;

// Raw pixels with their size upload when the scene loads.
let glow_bytes: Vec<u8> = vec![
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
];
let mat = Material::pbr()
    .emissive([0.8, 0.0, 0.0])
    .emissive_texture((glow_bytes, [2, 2]));
# let _ = mat;
```
