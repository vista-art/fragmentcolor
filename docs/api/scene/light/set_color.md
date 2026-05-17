# Light::set_color

Update the linear-RGB color the light emits. Defined for every kind —
returns `Self` for chaining. The new value propagates live to every
shader the Light has already been wired into; no re-attach needed.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let lamp = Light::point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0]);

// Warm-tint the lamp later — every Pass that absorbed `lamp` sees the
// new color on the next render.
lamp.set_color([1.0, 0.7, 0.4]);
# assert_eq!(lamp.color(), [1.0, 0.7, 0.4]);
# Ok(())
# }
```
