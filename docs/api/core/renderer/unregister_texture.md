# Renderer::unregister_texture(id)

Explicitly remove a texture from the renderer’s registry.

- Call this when you replace a texture, stop a video stream, or tear down a scene, to release GPU memory.
- If the texture is still referenced elsewhere, it will remain alive until all strong references are dropped.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let id = *renderer
    .create_storage_texture([16, 16], TextureFormat::Rgba, None)
    .await?
    .id();

renderer.unregister_texture(id)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
