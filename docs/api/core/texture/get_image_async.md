# Texture::get_image_async()

Async variant of [`Texture::get_image`] that works on both native and WASM.

Returns the mip-0 contents of this texture as tightly-packed bytes in the texture's native format.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture([64, 64], TextureFormat::Rgba, None).await?;
texture.write(&vec![0u8; 64 * 64 * 4])?;

let bytes = texture.get_image_async().await?;
# assert_eq!(bytes.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
