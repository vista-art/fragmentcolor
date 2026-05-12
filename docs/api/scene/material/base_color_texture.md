# Material::base_color_texture

Bind a texture to the canonical `base_color_map` slot. With a custom shader
that samples it, the per-fragment albedo is `material.base_color *
textureSample(base_color_map, sampler, in.uv)`.

The factors-only built-in PBR shader does not yet sample this slot — the
binding name is reserved so this call is forward-compatible with the
follow-up that adds texture sampling.

## Example

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let texture = renderer.create_texture(&[
    255u8, 200, 120, 255,
    255,  240, 180, 255,
    230,  180, 100, 255,
    255,  220, 150, 255,
][..]).await?;
let mat = Material::pbr().base_color_texture(&texture);
# let _ = mat;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
