# ToonMaterial::highlight

Set the color of the hard highlight, in linear RGB. Each light adds it, as a
spot with a hard edge, where the half vector between the light and the eye
comes within
[`highlight_size`](https://fragmentcolor.org/api/scene/toonmaterial#highlight_size)
of the normal, on faces turned toward the light. The light's color and
falloff scale it, as they scale the diffuse term.

Maps to the `material.highlight` uniform on the underlying shader. Default
is `[0, 0, 0]`, which adds nothing.

Returns a handle to the same material (Arc-shared), so calls chain.

An item that is NaN or an infinity returns `MaterialError::NotFinite`,
which names the call and the item, and the material keeps the value it
had. The other languages also refuse an item past the 32-bit float range,
such as `1e39`. JavaScript throws an `Error` named `InvalidInputError`,
Python raises `InvalidInputError`, Swift throws `FragmentColorError` and
Kotlin throws `FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::ToonMaterial;

let glossy = ToonMaterial::new().highlight([1.0, 1.0, 1.0])?;
# let _glossy = glossy;
# Ok(())
# }
```
