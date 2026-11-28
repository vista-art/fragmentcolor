# PbrMaterial::metallic

Set the metallic factor in `[0, 1]`. `0` is a pure dielectric (uses the
default F0 of 0.04), `1` is a pure metal (F0 = `base_color`). Values in
between linearly interpolate.

Maps to the `material.metallic` uniform. Default is `0.0` (dielectric);
`PbrMaterial::new` deviates from the glTF spec's `1.0` default here so the
out-of-the-box surface reads as a clean dielectric rather than dark gunmetal
under the factor-driven defaults.

NaN or an infinity returns `MaterialError::NotFinite`, which names the
call, and the material keeps the value it had. The other languages also
refuse a number past the 32-bit float range, such as `1e39`, with a
message like `PbrMaterial.metallic: value must be a finite number that fits a
32-bit float, got 1e39`. JavaScript throws an `Error` named
`InvalidInputError`, Python raises `InvalidInputError`, Swift throws
`FragmentColorError` and Kotlin throws `FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::PbrMaterial;

let chrome = PbrMaterial::new().metallic(1.0)?.roughness(0.05)?;
# let _chrome = chrome;
# Ok(())
# }
```
