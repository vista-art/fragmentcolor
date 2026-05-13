# Light::set_color

Update the linear-RGB color / intensity. The new value is written into
every Material that absorbed this Light via
[`Material::add`](https://fragmentcolor.org/api/scene/material#add).

Returns a handle to the same Light (Arc-shared) for chaining.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let lamp = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
// Warm-tinted bulb after the user toggles the warm-light switch.
lamp.set_color([1.0, 0.85, 0.7]);
# Ok(())
# }
```
