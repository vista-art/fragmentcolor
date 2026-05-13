# Light::directional

Construct a directional light. `direction` is the world-space vector the
light travels along (so `[0, -1, 0]` is straight down — noon sun); `color`
is linear-RGB intensity (`[1, 1, 1]` for full white, scaled values for
dimmer or tinted lights).

Use this for sun / moon / fill / key lights — anything where every shaded
surface should receive parallel rays from a fixed direction. Point and spot
lights aren't in the MVP yet; the module-level [`Light`](https://fragmentcolor.org/api/scene/light)
doc covers what's coming.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);

# assert_eq!(sun.direction(), [0.3, -1.0, -0.4]);
# assert_eq!(sun.color(), [1.0, 0.95, 0.9]);
# Ok(())
# }
```
