# Light::set_color

Update the linear-RGB color the light emits. Defined for every kind;
returns `Ok(self)` for chaining. The new value propagates live to every
shader the Light has already been wired into; no re-attach needed.

A channel that is NaN or an infinity returns `LightError::NotFinite`,
which names the call and the item, and the light keeps the color it had.
The other languages also refuse a channel past the 32-bit float range,
such as `1e39`, with a message like `Light.setColor: item 0 must be a
finite number that fits a 32-bit float, got 1e39`. JavaScript throws an
`Error` named `InvalidInputError`, Python raises `InvalidInputError`,
Swift throws `FragmentColorError` and Kotlin throws
`FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let lamp = Light::point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0])?;

// Warm-tint the lamp later. Every Pass that absorbed `lamp` sees the
// new color on the next render.
lamp.set_color([1.0, 0.7, 0.4])?;
# assert_eq!(lamp.color(), [1.0, 0.7, 0.4]);
# Ok(())
# }
```
