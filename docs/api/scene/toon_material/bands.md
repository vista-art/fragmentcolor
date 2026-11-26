# ToonMaterial::bands

Set how many even steps each light's diffuse term takes, from the edge of
the shadow to a face turned straight at the light. A face at the cosine
`ndl` to the light gets `ceil(ndl * bands) / bands` of it, and a face turned
away gets none, so the scene's ambient light alone fills it. One band lights
every face toward the light fully, and many bands come close to smooth
shading.

Maps to the `material.bands` uniform on the underlying shader. Default
is `3`. A [`ramp_map()`](https://fragmentcolor.org/api/scene/toonmaterial#ramp_map)
takes over from the bands once it holds a texture wider than one texel.

Returns a handle to the same material (Arc-shared), so calls chain.

`0` returns `MaterialError::Bands`, such as `ToonMaterial::bands takes 1
band or more, got 0`, and the material keeps the bands it had. JavaScript
throws an `Error` named `MaterialError`, Python raises `MaterialError`, a
`FragmentColorError` that is also a `ValueError`, Swift throws
`FragmentColorError`, and Kotlin throws `FragmentColorException`. The count
is a count like any other: JavaScript throws for a fraction or a negative
number, and Python takes an `int`, raising `TypeError` for a float and
`InvalidInputError` for an int below 0 or past `2**32 - 1`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::ToonMaterial;

// Lit or in shadow, and nothing between.
let flat = ToonMaterial::new().bands(1)?;
# let _flat = flat;
# Ok(())
# }
```
