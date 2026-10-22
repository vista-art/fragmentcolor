# Light::point

Construct a point light. `position` is the world-space location the light
radiates from; `color` is linear-RGB intensity (`[1, 1, 1]` for full white).
Falloff is inverse-square distance. Bring
[`set_range`](https://fragmentcolor.org/api/scene/light#lightset_range) in to
cap the influence radius, and
[`set_intensity`](https://fragmentcolor.org/api/scene/light#lightset_intensity)
to scale the color uniformly.

Use this for lamps, candles, fireballs: anything where light radiates
outward from a fixed point. For a parallel-ray sun-like light see
[`Light::directional`](https://fragmentcolor.org/api/scene/light#lightdirectional);
for a cone-constrained beam see
[`Light::spot`](https://fragmentcolor.org/api/scene/light#lightspot).

A point light carries no direction or cone. Calling
[`set_direction`](https://fragmentcolor.org/api/scene/light#lightset_direction)
or [`set_cone_angles`](https://fragmentcolor.org/api/scene/light#lightset_cone_angles)
on a point light returns `LightError::FieldNotApplicable`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let bulb = Light::point([0.0, 2.5, 0.0], [1.0, 0.95, 0.8]).set_intensity(15.0);

# assert_eq!(bulb.position(), Some([0.0, 2.5, 0.0]));
# assert_eq!(bulb.color(), [1.0, 0.95, 0.8]);
# assert!((bulb.intensity() - 15.0).abs() < 1.0e-6);
# Ok(())
# }
```
