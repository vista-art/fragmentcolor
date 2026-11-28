# ToonMaterial::rim_width

Set how far in from the silhouette the rim reaches, from `0`, which hides
it, to `1`, which covers every face not turned straight at the camera. The
rim covers the faces where the cosine between the normal and the direction
to the camera stays under the width. At the default `0.2`, it covers the
faces turned more than about 78 degrees from the camera.

Maps to the `material.rim_width` uniform on the underlying shader.

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

let glow = ToonMaterial::new()
    .rim([1.0, 0.6, 0.2])?
    .rim_width(0.35)?;
# let _glow = glow;
# Ok(())
# }
```
