# Material::metallic

Set the metallic factor in `[0, 1]`. `0` is a pure dielectric (uses the
default F0 of 0.04), `1` is a pure metal (F0 = `base_color`). Values in
between linearly interpolate.

Maps to the `material.metallic` uniform. Default is `0.0` (dielectric);
`Material::pbr` deviates from the glTF spec's `1.0` default here so the
out-of-the-box surface reads as a clean dielectric rather than dark gunmetal
under the factor-driven defaults.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let chrome = Material::pbr(&renderer).await?.metallic(1.0).roughness(0.05);
# let _ = chrome;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
