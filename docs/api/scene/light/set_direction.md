# Light::set_direction

Update the world-space travel direction. The new value is written into
every Material that absorbed this Light via
[`Material::add`](https://fragmentcolor.org/api/scene/material#add).

Returns a handle to the same Light (Arc-shared) for chaining.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
// Reorient to a late-afternoon angle.
sun.set_direction([0.7, -0.5, -0.5]);
# Ok(())
# }
```
