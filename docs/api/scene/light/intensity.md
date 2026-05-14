# Light::intensity

Read the scalar intensity multiplier. The shader uses `color * intensity *
attenuation` as the per-fragment radiance, so this is a cheap dimmer knob
that scales the visible brightness without disturbing the chromaticity.

Default is `1.0`. Matches glTF `KHR_lights_punctual`'s `intensity` field
(lumens for point/spot, lux for directional — interpretation is up to
your asset pipeline; the shader is unit-agnostic).

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let lamp = Light::point([0.0, 1.0, 0.0], [1.0, 0.95, 0.8]).set_intensity(12.0);
let scale = lamp.intensity();
# assert!((scale - 12.0).abs() < 1.0e-6);
# Ok(())
# }
```
