# Light::color

Read the linear-RGB color the light emits. Defined for every kind. The
returned value is the `[r, g, b]` triple stored on the Light. Call
[`set_color`](https://fragmentcolor.org/api/scene/light/set_color) to
update it. The scalar
[`intensity`](https://fragmentcolor.org/api/scene/light/intensity)
multiplier is separate and multiplies the color uniformly in the shader.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let warm_lamp = Light::point([0.0, 2.0, 0.0], [1.0, 0.7, 0.4]);
# assert_eq!(warm_lamp.color(), [1.0, 0.7, 0.4]);
# Ok(())
# }
```
