# Renderer::create_render_texture

Create a texture suitable as a per-pass render target that can also be sampled or read back later.

- Usage flags: RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_SRC

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};

let r = Renderer::new();
let tex = r.create_render_texture([256, 256], TextureFormat::Rgba8Unorm).await?;
# _ = tex;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```