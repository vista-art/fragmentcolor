# Light::intensity

Read the scalar intensity multiplier. Defined for every kind. The shader
multiplies the [`color`](https://fragmentcolor.org/api/scene/light/color)
by this value when accumulating the light's contribution, so doubling
intensity doubles every channel uniformly. Call
[`set_intensity`](https://fragmentcolor.org/api/scene/light/set_intensity)
to update it.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let bright = Light::point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0]).set_intensity(5.0);
# assert!((bright.intensity() - 5.0).abs() < 1.0e-6);
# Ok(())
# }
```
