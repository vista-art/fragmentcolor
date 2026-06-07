# Light::set_intensity

Update the scalar intensity multiplier. Defined for every kind; returns
`Self` for chaining. The shader multiplies the
[`color`](https://fragmentcolor.org/api/scene/light/color) by this value
when accumulating, so doubling intensity doubles every channel uniformly.
The new value propagates live to every shader the Light has already been
wired into.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let torch = Light::spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);

torch.set_intensity(8.0);
# assert!((torch.intensity() - 8.0).abs() < 1.0e-6);
# Ok(())
# }
```
