# Light::inner_cone_angle

Read the inner cone half-angle (radians). Fragments inside this cone
receive the light at full intensity; between the inner and outer cone
the contribution falls off smoothly to zero.

Only meaningful for [`Light::spot`](https://fragmentcolor.org/api/scene/light/spot).
Defaults to `0.0` (single-ray center). Matches glTF
`KHR_lights_punctual`'s `spot.innerConeAngle`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let torch = Light::spot([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0])
    .set_cone_angles(0.2, 0.5);
let inner = torch.inner_cone_angle();
# assert!((inner - 0.2).abs() < 1.0e-6);
# Ok(())
# }
```
