# Material::emissive_texture

Bind an emissive map to the canonical `emissive_map` slot. The default PBR
shader samples it in `fs_main` and multiplies by the factor: per-fragment
emission is `material.emissive * textureSample(emissive_map, sampler, in.uv).rgb`.

Unset, this slot resolves to a 1×1 white default so the multiplied
emission falls back to the `material.emissive` factor as-is.

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
let mat = Material::pbr()?
    .emissive([0.8, 0.0, 0.0])
    .emissive_texture(&glow);
# let _ = mat;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
