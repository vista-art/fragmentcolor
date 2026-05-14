# Light::set_intensity

Update the scalar intensity multiplier. The shader uses
`color * intensity * attenuation` as the per-fragment radiance, so this
scales the brightness without disturbing the chromaticity.

Returns a handle to the same Light (Arc-shared) for chaining. Negative
values are accepted and stored verbatim; they produce a "negative light"
that subtracts radiance, which is occasionally useful for shadowed-zone
hacks but generally not what you want.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let lamp = Light::point([0.0, 1.0, 0.0], [1.0, 0.95, 0.8]);
lamp.set_intensity(15.0);
# assert!((lamp.intensity() - 15.0).abs() < 1.0e-6);
# Ok(())
# }
```
