# Quad::new()

Create a quad from two corners (min.xy, max.xy) in clip-space coordinates.

Each corner holds 2 numbers. In JavaScript, Python, Swift and Kotlin, another count, or a number that is NaN, infinite or past the 32-bit float range, is an error that names the corner.

- Position layout: @location(0) vec2<f32>
- UV layout: @location(1) vec2<f32>

## Example

```rust
use fragmentcolor::Quad;

let quad = Quad::new([-0.5, -0.5], [0.5, 0.5]);

# _ = quad;
```
