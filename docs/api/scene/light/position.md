# Light::position

Read the world-space position the light radiates from. Returns
`Some([x, y, z])` for a point or spot light, `None` for a directional
light (directional rays travel from infinitely far away, so there's no position to
report). Call
[`set_position`](https://fragmentcolor.org/api/scene/light/set_position)
to update it.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let lamp = Light::point([0.0, 2.5, 0.0], [1.0, 1.0, 1.0]);
let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
# assert_eq!(lamp.position(), Some([0.0, 2.5, 0.0]));
# assert!(sun.position().is_none());
# Ok(())
# }
```
