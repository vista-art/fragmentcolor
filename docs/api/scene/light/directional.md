# Light::directional

Construct a directional light. `direction` is the world-space vector the
light travels along (so `[0, -1, 0]` is straight down, the noon sun); `color`
is linear-RGB intensity (`[1, 1, 1]` for full white, scaled values for
dimmer or tinted lights).

Use this for sun / moon / fill / key lights: anything where every shaded
surface should receive parallel rays from a fixed direction. For positioned
sources, see [`Light::point`](https://fragmentcolor.org/api/scene/light#point)
and [`Light::spot`](https://fragmentcolor.org/api/scene/light#spot).

A directional light carries no position. Calling
[`set_position`](https://fragmentcolor.org/api/scene/light#set_position),
[`set_range`](https://fragmentcolor.org/api/scene/light#set_range), or
[`set_cone_angles`](https://fragmentcolor.org/api/scene/light#set_cone_angles)
on a directional light returns `LightError::FieldNotApplicable`.

An item of `direction` or `color` that is NaN or an infinity returns
`LightError::NotFinite`, which names the call, the vector and the item:
`Light::directional: direction: item 1 must be finite, got NaN`. The other
languages also refuse an item past the 32-bit float range, such as `1e39`,
with a message like `Light.directional: direction: item 1 must be a finite
number that fits a 32-bit float, got 1e39`. JavaScript throws an `Error`
named `InvalidInputError`, Python raises `InvalidInputError`, Swift throws
`FragmentColorError` and Kotlin throws `FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])?;
# assert_eq!(sun.direction(), Some([0.3, -1.0, -0.4]));
# assert_eq!(sun.color(), [1.0, 0.95, 0.9]);
# assert!(sun.position().is_none());
# Ok(())
# }
```
