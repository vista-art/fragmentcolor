# Material::occlusion_texture

Bind an ambient-occlusion map to the canonical `occlusion_map` slot.
Following glTF 2.0, only the red channel is read; the sampled value is
blended toward `1.0` by `1 - material.occlusion_strength` inside the
default PBR shader, then multiplied into the diffuse term.

Takes a [`Texture`](https://fragmentcolor.org/api/texture/texture), or bytes, a path, a URL
or a `Mipmap`, which the Material keeps as a map that each `Renderer` uploads
a copy of. [`base_color_texture`](https://fragmentcolor.org/api/scene/material#base_color_texture)
tells how a map loads, what it holds in memory and how it fails.

Unset, this slot resolves to a 1×1 white default so the multiplied
occlusion factor is `1.0` and no darkening is applied.

## Example

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let ao_bytes: Vec<u8> = vec![
    220, 0, 0, 255,
    180,   0, 0, 255,
    200,   0, 0, 255,
    160,   0, 0, 255,
];
let ao = renderer.create_texture((ao_bytes, [2, 2])).await?;
let mat = Material::pbr().occlusion_texture(&ao);
# let _ = mat;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
