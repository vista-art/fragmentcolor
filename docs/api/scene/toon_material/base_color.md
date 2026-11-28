# ToonMaterial::base_color

Set the material's color, in linear RGBA. The base color map's sample and
the vertex color multiply it, each light's band or ramp level scales it, and
alpha goes through the fragment output unchanged.

Maps to the `material.base_color` uniform on the underlying shader. Default
is `[1, 1, 1, 1]`.

Returns a handle to the same material (Arc-shared), so calls chain.

An item that is NaN or an infinity returns `MaterialError::NotFinite`,
which names the call and the item, and the material keeps the value it
had. The other languages also refuse an item past the 32-bit float range,
such as `1e39`, with a message like `ToonMaterial.baseColor: item 3 must be a
finite number that fits a 32-bit float, got 1e39`. JavaScript throws an
`Error` named `InvalidInputError`, Python raises `InvalidInputError`,
Swift throws `FragmentColorError` and Kotlin throws
`FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::ToonMaterial;

let red = ToonMaterial::new().base_color([1.0, 0.2, 0.2, 1.0])?;
# let _ = red;
# Ok(())
# }
```
