# Material::normal_texture

Bind a tangent-space normal map to the canonical `normal_map` slot. The
sampled XY normal is reconstructed in the shader and scaled by
`material.normal_scale`.

The factors-only built-in PBR shader does not yet sample this slot — the
binding name is reserved so this call is forward-compatible.

## Example

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let normal_map = renderer.create_texture(&[
    128u8, 128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
][..]).await?;
let mat = Material::pbr().normal_texture(&normal_map).normal_scale(1.2);
# let _ = mat;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
