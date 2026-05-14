# Light::range

Read the maximum influence radius (world units). `0.0` means unlimited —
the inverse-square falloff still applies but is never clipped. Any value
above zero adds a smooth cutoff: the contribution rolls off and reaches
zero at exactly `range`.

Only meaningful for [`Light::point`](https://fragmentcolor.org/api/scene/light/point)
and [`Light::spot`](https://fragmentcolor.org/api/scene/light/spot). Matches
glTF `KHR_lights_punctual`'s `range`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let bulb = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).set_range(8.0);
let cutoff = bulb.range();
# assert!((cutoff - 8.0).abs() < 1.0e-6);
# Ok(())
# }
```
