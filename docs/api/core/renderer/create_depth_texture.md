# Renderer::create_depth_texture

Create a depth texture using `Depth32Float`.

A pass that draws models gets a depth buffer without one. Create a depth texture to read depth in a later pass, or to give depth to a pass of plain meshes.

A width or height of zero, a side past the device limit, or a depth other than 1 is an error, the one `resize` returns: `TextureError::InvalidInput` inside `InitializationError::TextureError` in Rust.

The created depth texture inherits the renderer's current sample count:
- If you called `create_target(window)` (surface-backed), it matches the negotiated MSAA (e.g., 2×/4×) for that surface.
- If you are rendering offscreen via `create_texture_target`, it defaults to 1.

This ensures the depth attachment `sample_count` matches the pass sample count.
A depth texture with a different `sample_count` than the pass makes `render` return
`RendererError::DepthSampleCountMismatch`, and one whose width or height differs from
the pass's color attachment returns `RendererError::DepthSizeMismatch`, before it takes
a frame. A window that resizes needs a depth texture of the new size, or a `resize` of this one.

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
