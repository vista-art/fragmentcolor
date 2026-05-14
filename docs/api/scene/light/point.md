# Light::point

Construct a point light. `position` is the world-space location the light
radiates from; `color` is linear-RGB intensity (`[1, 1, 1]` for full white).
Falloff is inverse-square distance — bring [`set_range`](https://fragmentcolor.org/api/scene/light/set_range)
in to cap the influence radius, and [`set_intensity`](https://fragmentcolor.org/api/scene/light/set_intensity)
to scale the color uniformly.

Use this for lamps, candles, fireballs — anything where light radiates
outward from a fixed point. The cone-angle fields default to `(0, π/4)`
but are ignored for point lights; they only matter when you upgrade the
light to a [`Light::spot`](https://fragmentcolor.org/api/scene/light/spot).

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let bulb = Light::point([0.0, 2.5, 0.0], [1.0, 0.95, 0.8]).set_intensity(15.0);

# assert_eq!(bulb.position(), [0.0, 2.5, 0.0]);
# assert_eq!(bulb.color(), [1.0, 0.95, 0.8]);
# assert!((bulb.intensity() - 15.0).abs() < 1.0e-6);
# Ok(())
# }
```
