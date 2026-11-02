# Material::roughness

Set the surface roughness in `[0, 1]`. `0` is a perfect mirror; `1` is a
fully matte surface. The PBR shader uses the squared form (alpha = r²) for
the GGX normal distribution, which is the GGX/Trowbridge-Reitz convention
shared with glTF 2.0.

Maps to the `material.roughness` uniform. Default is `1.0`.

NaN or an infinity returns `MaterialError::NotFinite`, which names the
call, and the material keeps the value it had. The other languages also
refuse a number past the 32-bit float range, such as `1e39`, with a
message like `Material.roughness: value must be a finite number that fits
a 32-bit float, got 1e39`. JavaScript throws an `Error` named
`InvalidInputError`, Python raises `InvalidInputError`, Swift throws
`FragmentColorError` and Kotlin throws `FragmentColorException`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let satin = Material::pbr().roughness(0.35)?;
# let _ = satin;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
