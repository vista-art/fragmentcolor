# Light::outer_cone_angle

Read the outer cone half-angle (radians). Beyond this angle from the cone
axis (`-direction`), the light contributes zero. Between the inner and
outer cone, the contribution falls off smoothly.

Only meaningful for [`Light::spot`](https://fragmentcolor.org/api/scene/light/spot).
Defaults to `π / 4` (a 45° outer cone). Matches glTF
`KHR_lights_punctual`'s `spot.outerConeAngle`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let torch = Light::spot([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
    .set_cone_angles(0.2, 0.5);
let outer = torch.outer_cone_angle();
# assert!((outer - 0.5).abs() < 1.0e-6);
# Ok(())
# }
```
