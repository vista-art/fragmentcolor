# Light::set_direction

Update the world-space direction. Defined for directional and spot lights
only; returns `Ok(self)` for chaining. Calling this on a point light
returns `Err(LightError::FieldNotApplicable { kind: Point, field:
"set_direction" })` because point sources radiate omnidirectionally. The
new value propagates live to every shader the Light has already been
wired into; no re-attach needed.

An item that is NaN or an infinity returns `LightError::NotFinite`, which
names the call and the item, on any kind, and the light keeps the
direction it had. The other languages also refuse an item past the 32-bit
float range, such as `1e39`. JavaScript throws an `Error` named
`InvalidInputError`, Python raises `InvalidInputError`, Swift throws
`FragmentColorError` and Kotlin throws `FragmentColorException`. A
direction of length 0, such as `[0, 0, 0]`, points the light nowhere and
returns `LightError::ZeroDirection`, which JavaScript and Python raise as
`InvalidInputError` too, and the light keeps its direction.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])?;
sun.set_direction([0.3, -0.8, -0.5])?;
# assert_eq!(sun.direction(), Some([0.3, -0.8, -0.5]));
#
# // Point lights have no direction; the call errors.
# let lamp = Light::point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0])?;
# let result = lamp.set_direction([0.0, -1.0, 0.0]);
# assert!(result.is_err());
# Ok(())
# }
```
