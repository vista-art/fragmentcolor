# Light::set_cone_angles

Update the inner and outer cone half-angles (radians). Inside the inner
cone the light contributes at full intensity; between inner and outer
the contribution falls off smoothly to zero; beyond the outer cone the
light contributes nothing.

Returns a handle to the same Light (Arc-shared) for chaining. Only
[`Light::spot`](https://fragmentcolor.org/api/scene/light/spot) consults
these values; directional and point lights store but ignore them.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let torch = Light::spot([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
torch.set_cone_angles(0.15, 0.5);
# assert!((torch.inner_cone_angle() - 0.15).abs() < 1.0e-6);
# assert!((torch.outer_cone_angle() - 0.5).abs() < 1.0e-6);
# Ok(())
# }
```
