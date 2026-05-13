# Light::direction

Read the world-space direction the light travels along, as `[x, y, z]`.
`[0, -1, 0]` is straight down (noon sun); `[0, 1, 0]` is straight up (the
light is coming from below).

The vector is whatever was passed to
[`Light::directional`](https://fragmentcolor.org/api/scene/light/directional);
it isn't normalized here. Shaders that need a unit vector normalize at
sample time.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 1.0, 1.0]);
let dir = sun.direction();
# assert_eq!(dir, [0.3, -1.0, -0.4]);
# Ok(())
# }
```
