# Material::emissive_texture

Bind an emissive map to the canonical `emissive_map` slot. The default PBR
shader samples it in `fs_main` and multiplies by the factor: per-fragment
emission is `material.emissive * textureSample(emissive_map, pbr_sampler, uv).rgb`,
where `uv` is `slot_uv(material.uv[4], in.uv0, in.uv1)`, the slot's own UV
set through its own transform.

Takes a [`Texture`](https://fragmentcolor.org/api/texture/texture), or bytes, a path, a URL
or a `Mipmap`, which the Material keeps as a map that each `Renderer` uploads
a copy of. [`base_color_texture`](https://fragmentcolor.org/api/scene/material#base_color_texture)
tells how a map loads, what it holds in memory and how it fails.

Unset, this slot resolves to a 1×1 white default so the multiplied
emission falls back to the `material.emissive` factor as-is.

## Example

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let glow_bytes: Vec<u8> = vec![
    255, 0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
];
let glow = renderer.create_texture((glow_bytes, [2, 2])).await?;
let mat = Material::pbr()
    .emissive([0.8, 0.0, 0.0])
    .emissive_texture(&glow);
# let _ = mat;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
