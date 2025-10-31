# Texture::id()

Return the stable `TextureId` for this texture instance. The id is valid within the `Renderer` that created it.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture([64, 64], TextureFormat::Rgba, None).await?;
let id = *texture.id();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
