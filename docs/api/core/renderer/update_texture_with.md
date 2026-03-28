# Renderer::update_texture_with(id, bytes, options)

Same as `Renderer::update_texture`, but allows specifying origin, size, and layout.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat, TextureWriteOptions};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture([64, 32], TextureFormat::Rgba, None).await?;
let id = *texture.id();
let frame = vec![0u8; 64 * 32 * 4];
let opt = TextureWriteOptions::whole().with_bytes_per_row(256).with_rows_per_image(32);

renderer.update_texture_with(id, &frame, opt)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
``
