# UnlitMaterial::alpha_cutoff

Sets the alpha threshold `AlphaMode::Mask` uses. The fragment shader
discards a fragment whose alpha, the base color's times the map's and the
vertex color's, falls below it. Stored on `material.alpha_cutoff`; default
`0.5`, matching glTF 2.0. `Opaque` and `Blend` ignore it.

NaN or an infinity returns `MaterialError::NotFinite`, which names the
call, and the material keeps the value it had. The other languages also
refuse a number past the 32-bit float range, such as `1e39`, with a
message like `UnlitMaterial.alphaCutoff: value must be a finite number that
fits a 32-bit float, got 1e39`. JavaScript throws an `Error` named
`InvalidInputError`, Python raises `InvalidInputError`, Swift throws
`FragmentColorError` and Kotlin throws `FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{AlphaMode, UnlitMaterial};

let sprite = UnlitMaterial::new()
    .alpha_mode(AlphaMode::Mask)
    .alpha_cutoff(0.3)?;
# let _sprite = sprite;
# Ok(())
# }
```
