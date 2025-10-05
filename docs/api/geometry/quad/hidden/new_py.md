# Quad::new()

Create a quad from two corners (min.xy, max.xy) in clip-space coordinates.

- Position layout: @location(0) vec2<f32>
- UV layout: @location(1) vec2<f32>

## Example

```rust
use fragmentcolor::Quad;

let quad = Quad::new([-0.5, -0.5], [0.5, 0.5]);

# _ = quad;
```
