# Light::inner_cone_angle

Read the inner cone half-angle (radians). Returns `Some(value)` only for a
spot light, `None` for directional and point lights. The inner cone is the
band where the light reaches full intensity; beyond it the contribution
smoothly rolls off until it reaches zero at the outer cone (see
[`outer_cone_angle`](https://fragmentcolor.org/api/scene/light/outer_cone_angle)).
Call
[`set_cone_angles`](https://fragmentcolor.org/api/scene/light/set_cone_angles)
to update both at once.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let torch = Light::spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
    .set_cone_angles(0.15, 0.4)?;
let lamp = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
# assert!((torch.inner_cone_angle().unwrap() - 0.15).abs() < 1.0e-6);
# assert!(lamp.inner_cone_angle().is_none());
# Ok(())
# }
```
