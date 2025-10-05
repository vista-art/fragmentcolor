# Quad

A simple rectangle (quad) builder for 2D rendering.
Produces a Mesh with position (vec2) and uv (vec2) per-vertex.

## Example

```rust
use fragmentcolor::Quad;

let quad = Quad::new([-0.5, -0.5], [0.5, 0.5]);

# _ = quad;
```
