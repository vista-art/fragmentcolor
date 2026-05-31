# Light::outer_cone_angle

Read the outer cone half-angle (radians). Returns `Some(value)` only for a
spot light, `None` for directional and point lights. The outer cone is
where the light's contribution reaches zero. Beyond this angle the
fragment receives nothing from this light. Between the inner and outer
cones the contribution smoothly rolls off; see
[`inner_cone_angle`](https://fragmentcolor.org/api/scene/light/inner_cone_angle).
Call
[`set_cone_angles`](https://fragmentcolor.org/api/scene/light/set_cone_angles)
to update both at once.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let torch = Light::spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
    .set_cone_angles(0.15, 0.4)?;
let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
# assert!((torch.outer_cone_angle().unwrap() - 0.4).abs() < 1.0e-6);
# assert!(sun.outer_cone_angle().is_none());
# Ok(())
# }
```
