# Texture::sample_count

Return the texture's MSAA sample count.

- 1 means single-sampled
- 2, 4, ... means multi-sampled attachment

Depth textures created via Renderer::create_depth_texture inherit the renderer's current sample count (surface MSAA when rendering to a window; 1 for offscreen texture targets).

## Example

```rust
use fragmentcolor::Renderer;
let r = Renderer::new();
let depth = r.create_depth_texture([640, 480]);
let sc = depth.sample_count();
assert!(sc >= 1);
```
