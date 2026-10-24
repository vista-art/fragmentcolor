# Renderer::create_depth_texture

Create a depth texture using `Depth32Float`.

The created depth texture inherits the renderer's current sample count:
- If you called `create_target(window)` (surface-backed), it matches the negotiated MSAA (e.g., 2×/4×) for that surface.
- If you are rendering offscreen via `create_texture_target`, it defaults to 1.

This ensures the depth attachment `sample_count` matches the pass sample count.
If you attach a depth texture with a different `sample_count` than the pass,
rendering will return a descriptive validation error.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Renderer;
let r = Renderer::new();
let depth = r.create_depth_texture([800, 600]).await?;
# _ = depth;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
