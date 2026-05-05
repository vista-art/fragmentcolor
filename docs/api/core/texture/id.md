# Texture::id()

Return the stable `TextureId` for this texture. The id is valid within
the `Renderer` that created it and is the handle uniforms use to refer to
the texture, so you can keep one around to bind, look up, or unregister
the texture without holding the full `Texture` value.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture(([64, 64], TextureFormat::Rgba)).await?;
let id = *texture.id();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
