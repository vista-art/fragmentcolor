# Material::emissive

Set the emissive factor (linear RGB): light the surface emits regardless of
incident illumination. Added on top of the PBR shading and clamped only by
the output format; for tonemapped pipelines use values in `[0, 1]`, for HDR
you can go beyond.

Maps to the `material.emissive` uniform. Default is `[0, 0, 0]`.

An item that is NaN or an infinity returns `MaterialError::NotFinite`,
which names the call and the item, and the material keeps the value it
had. The other languages also refuse an item past the 32-bit float range,
such as `1e39`, with a message like `Material.emissive: item 1 must be a
finite number that fits a 32-bit float, got 1e39`. JavaScript throws an
`Error` named `InvalidInputError`, Python raises `InvalidInputError`,
Swift throws `FragmentColorError` and Kotlin throws
`FragmentColorException`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let lava = Material::pbr()
    .base_color([0.1, 0.05, 0.0, 1.0])?
    .emissive([1.5, 0.4, 0.1])?;
# let _ = lava;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
