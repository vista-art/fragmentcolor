# ToonMaterial::rim

Set the color of the rim, in linear RGB, which the material adds to the
faces the camera sees edge-on, along the silhouette. It needs no light, so it
shows in shadow too. [`rim_width`](https://fragmentcolor.org/api/scene/toonmaterial#rim_width)
sets how far in from the edge it reaches.

Maps to the `material.rim` uniform on the underlying shader. Default is
`[0, 0, 0]`, which adds nothing.

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

let ghost = ToonMaterial::new().rim([0.4, 0.8, 1.0])?;
# let _ghost = ghost;
# Ok(())
# }
```
