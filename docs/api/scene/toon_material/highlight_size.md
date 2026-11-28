# ToonMaterial::highlight_size

Set how large the highlight's spot is, from `0`, which hides it, to `1`,
which spreads it over every face turned toward the light. The spot covers
the faces where the cosine between the normal and the half vector, between
the light and the eye, tops `1 - size`. At the default `0.05`, the spot
reaches about 18 degrees from the glint.

Maps to the `material.highlight_size` uniform on the underlying shader.

Returns a handle to the same material (Arc-shared), so calls chain.

NaN or an infinity returns `MaterialError::NotFinite`, which names the
call, and the material keeps the value it had. The other languages also
refuse a number past the 32-bit float range, such as `1e39`. JavaScript
throws an `Error` named `InvalidInputError`, Python raises
`InvalidInputError`, Swift throws `FragmentColorError` and Kotlin throws
`FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::ToonMaterial;

let wet = ToonMaterial::new()
    .highlight([1.0, 1.0, 1.0])?
    .highlight_size(0.15)?;
# let _wet = wet;
# Ok(())
# }
```
