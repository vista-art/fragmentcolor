# Light::direction

Read the world-space direction the light travels along. Returns
`Some([x, y, z])` for a directional or spot light, `None` for a point
light (point sources radiate omnidirectionally, so there's no direction to report).
Call
[`set_direction`](https://fragmentcolor.org/api/scene/light/set_direction)
to update it.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 1.0, 1.0]);
let lamp = Light::point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0]);
# assert_eq!(sun.direction(), Some([0.3, -1.0, -0.4]));
# assert!(lamp.direction().is_none());
# Ok(())
# }
```
