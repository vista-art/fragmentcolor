# Renderer::unregister_texture(id)

Remove a texture from the renderer’s registry now.

The renderer releases a texture on its own when the last thing holding it
drops: every `Texture` handle, the `TextureTarget` that owns it, and each
shader uniform or pass target that binds it. Call this to release one sooner,
for example when a video stream stops.

- After the call, the id names no texture, and shaders and passes that bind it can't draw it. A pass that keeps it as its color or depth target makes `render` return `TextureNotFoundError`.
- A `Texture` handle you still hold keeps its GPU memory until it drops.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture(([16, 16], TextureFormat::Rgba)).await?;
let id = texture.id();

renderer.unregister_texture(id)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
