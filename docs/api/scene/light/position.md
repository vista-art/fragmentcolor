# Light::position

Read the world-space position as `[x, y, z]`. Meaningful for
[`Light::point`](https://fragmentcolor.org/api/scene/light/point) and
[`Light::spot`](https://fragmentcolor.org/api/scene/light/spot); ignored
by the shader for directional lights, where rays are parallel and the
position has no physical meaning.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let bulb = Light::point([3.0, 2.5, -1.0], [1.0, 1.0, 1.0]);
let pos = bulb.position();
# assert_eq!(pos, [3.0, 2.5, -1.0]);
# Ok(())
# }
```
