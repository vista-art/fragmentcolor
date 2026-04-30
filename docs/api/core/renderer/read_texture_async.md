# Renderer::read_texture_async(texture_id)

Async variant of [`Renderer::read_texture`] that works on both native and WASM.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture([64, 64], TextureFormat::Rgba, None).await?;
texture.write(&vec![0u8; 64 * 64 * 4])?;

let bytes = renderer.read_texture_async(*texture.id()).await?;
# assert_eq!(bytes.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
