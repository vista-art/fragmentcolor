# Light::set_range

Update the maximum influence radius (world units). Negative values are
clamped to `0.0` (unlimited). Any value above zero adds a smooth cutoff
that drops the contribution to zero at exactly `range`.

Returns a handle to the same Light (Arc-shared) for chaining. Only
[`Light::point`](https://fragmentcolor.org/api/scene/light/point) and
[`Light::spot`](https://fragmentcolor.org/api/scene/light/spot) consult
this value.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Light;

let bulb = Light::point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
bulb.set_range(8.0);
# assert!((bulb.range() - 8.0).abs() < 1.0e-6);
# Ok(())
# }
```
