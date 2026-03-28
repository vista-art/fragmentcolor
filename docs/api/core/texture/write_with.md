# Texture::write_with(bytes, options)

Same as `Texture::write`, but allows specifying origin, size, bytes_per_row and rows_per_image.

## Notes
- See `Texture::write` for format and alignment details.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat, TextureWriteOptions};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture([64, 32], TextureFormat::Rgba, None).await?;
let region_bytes = vec![0u8; 64 * 32 * 4];
let opt = TextureWriteOptions::whole().with_bytes_per_row(256).with_rows_per_image(32);
texture.write_with(&region_bytes, opt)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
``
