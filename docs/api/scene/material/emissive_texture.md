# Material::emissive_texture

Bind an emissive map to the canonical `emissive_map` slot. With a custom
shader that samples it, the per-fragment emission is `material.emissive *
textureSample(emissive_map, sampler, in.uv).rgb`.

The factors-only built-in PBR shader does not yet sample this slot — the
binding name is reserved so this call is forward-compatible.

## Example

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let glow = renderer.create_texture(&[
    255u8, 0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
    255,   0, 0, 255,
][..]).await?;
let mat = Material::pbr()
    .emissive([0.8, 0.0, 0.0])
    .emissive_texture(&glow);
# let _ = mat;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
