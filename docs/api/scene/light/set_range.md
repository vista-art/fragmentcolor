# Light::set_range

Update the influence-radius cap. Defined for point and spot lights only —
returns `Ok(self)` for chaining. Calling this on a directional light
returns `Err(LightError::FieldNotApplicable { kind: Directional, field:
"set_range" })` because parallel rays have no distance falloff. Passing a
negative value returns `Err(LightError::NegativeRange(value))` regardless
of kind.

A value of `0.0` means "unlimited", matching glTF `KHR_lights_punctual`'s
default; positive values cut the contribution off smoothly past that
distance. The new value propagates live to every shader the Light has
already been wired into.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let lamp = Light::point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0]);
lamp.set_range(8.0)?;
let negative = lamp.set_range(-1.0);

// Directional lights have no range — the call errors.
let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
let unsupported = sun.set_range(5.0);
# assert_eq!(lamp.range(), Some(8.0));
# assert!(negative.is_err());
# assert!(unsupported.is_err());
# Ok(())
# }
```
