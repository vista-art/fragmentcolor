# Vertex::position_at(index: u32)

Planned API (draft)

Pin the vertex position attribute to a specific shader location `@location(index)`. This is optional; by default, position occupies location 0 and is exposed as the `position` attribute with a 2‑ or 3‑component format depending on how the vertex was constructed.

Notes
- This does not change the fact that WGSL’s `@builtin(position)` is a vertex shader output; this API only controls the vertex buffer input location for the position attribute.
- Use with caution: ensure your shader’s vertex input at `@location(index)` expects a compatible format (vec2<f32> or vec3<f32>).

## Example

```rust
use fragmentcolor::mesh::Vertex;

let mut v = Vertex::new([0.0f32, 0.0, 0.0]);
// Place position at location 3 to match a custom shader layout
v.position_at(3);
```