# Light::range

Read the influence-radius cap. Returns `Some(value)` for a point or spot
light, `None` for a directional light (parallel rays have no distance
falloff). A value of `0.0` means "unlimited", matching glTF
`KHR_lights_punctual`'s default; positive values cut the contribution off
smoothly past that distance. Call
[`set_range`](https://fragmentcolor.org/api/scene/light/set_range) to
update it.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let lamp = Light::point([0.0, 2.0, 0.0], [1.0, 1.0, 1.0]);
let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
# assert_eq!(lamp.range(), Some(0.0));
# assert!(sun.range().is_none());
# Ok(())
# }
```
