# Material::emissive

Set the emissive factor (linear RGB) — light the surface emits regardless of
incident illumination. Added on top of the PBR shading and clamped only by
the output format; for tonemapped pipelines use values in `[0, 1]`, for HDR
you can go beyond.

Maps to the `material.emissive` uniform. Default is `[0, 0, 0]`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let lava = Material::pbr()?
    .base_color([0.1, 0.05, 0.0, 1.0])
    .emissive([1.5, 0.4, 0.1]);
# let _ = lava;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
