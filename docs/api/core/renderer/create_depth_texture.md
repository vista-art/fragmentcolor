# Renderer::create_depth_texture

Create a depth texture using `Depth32Float`.

## Example

```rust
# use fragmentcolor::Renderer;
let r = Renderer::new();
let depth = r.create_depth_texture([800, 600]);
```
