# Light::spot

Construct a spot light. `position` is the world-space location the light
radiates from; `direction` is the world-space vector the cone aims along
(the cone axis is `-direction`); `color` is linear-RGB intensity.

Falloff is inverse-square distance with a smooth cone roll-off between an
inner and outer cone angle (defaults: `0` and `π/4`). Tighten the beam with
[`set_cone_angles`](https://fragmentcolor.org/api/scene/light/set_cone_angles),
cap the influence with [`set_range`](https://fragmentcolor.org/api/scene/light/set_range),
and scale the radiance with [`set_intensity`](https://fragmentcolor.org/api/scene/light/set_intensity).

Use this for flashlights, headlamps, theatre lighting, mood key-lights —
anything where the lit volume is a cone aimed at a target.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let torch = Light::spot([0.0, 1.8, 1.0], [0.0, -0.3, -1.0], [1.0, 0.9, 0.7])
    .set_intensity(5.0)
    .set_cone_angles(0.15, 0.4);

# assert_eq!(torch.position(), [0.0, 1.8, 1.0]);
# assert_eq!(torch.direction(), [0.0, -0.3, -1.0]);
# assert!((torch.outer_cone_angle() - 0.4).abs() < 1.0e-6);
# Ok(())
# }
```
