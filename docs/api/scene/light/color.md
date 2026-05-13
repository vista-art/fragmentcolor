# Light::color

Read the light's linear-RGB color / intensity as `[r, g, b]`. `[1, 1, 1]`
is full white at unit intensity; values above 1.0 boost the contribution
(useful for HDR pipelines or very-bright key lights), values below 1.0 dim
or tint the light.

Not premultiplied, not gamma-corrected: shaders multiply diffuse + specular
response by this value directly under the assumption it lives in linear
light-space.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let warm = Light::directional([0.0, -1.0, 0.0], [1.0, 0.85, 0.7]);
let color = warm.color();
# assert_eq!(color, [1.0, 0.85, 0.7]);
# Ok(())
# }
```
