# Light::set_position

Update the world-space position. Defined for point and spot lights only;
returns `Ok(self)` for chaining. Calling this on a directional light
returns `Err(LightError::FieldNotApplicable { kind: Directional, field:
"set_position" })` because directional rays carry no position. The new
value propagates live to every shader the Light has already been wired
into; no re-attach needed.

An item that is NaN or an infinity returns `LightError::NotFinite`, which
names the call and the item, on any kind, and the light stays where it
was. The other languages also refuse an item past the 32-bit float range,
such as `1e39`. JavaScript throws an `Error` named `InvalidInputError`,
Python raises `InvalidInputError`, Swift throws `FragmentColorError` and
Kotlin throws `FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let lamp = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])?;
lamp.set_position([3.0, 1.5, -2.0])?;
# assert_eq!(lamp.position(), Some([3.0, 1.5, -2.0]));
#
# // Directional lights have no position; the call errors.
# let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0])?;
# let result = sun.set_position([0.0, 0.0, 0.0]);
# assert!(result.is_err());
# Ok(())
# }
```
