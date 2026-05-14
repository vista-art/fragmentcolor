# Light::set_position

Update the world-space position. The new value is written into every
Material that absorbed this Light via [`Pass::add`](https://fragmentcolor.org/api/core/pass#add).

Returns a handle to the same Light (Arc-shared) for chaining. Only
[`Light::point`](https://fragmentcolor.org/api/scene/light/point) and
[`Light::spot`](https://fragmentcolor.org/api/scene/light/spot) consult
this value; directional lights store but ignore it.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let bulb = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
bulb.set_position([2.0, 3.0, -1.0]);
# assert_eq!(bulb.position(), [2.0, 3.0, -1.0]);
# Ok(())
# }
```
