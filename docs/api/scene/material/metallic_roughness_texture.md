# Material::metallic_roughness_texture

Bind a texture to the canonical `metallic_roughness_map` slot. Following
glTF 2.0, the green channel encodes per-fragment roughness, the blue channel
encodes metallic — both multiplied by their respective factors at sample
time.

The factors-only built-in PBR shader does not yet sample this slot — the
binding name is reserved so this call is forward-compatible.

## Example

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let mr_map = renderer.create_texture(&[
    0u8, 200, 50, 255,
    0,   240, 80, 255,
    0,   180, 30, 255,
    0,   220, 60, 255,
][..]).await?;
let mat = Material::pbr().metallic_roughness_texture(&mr_map);
# let _ = mat;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
