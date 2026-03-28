# Texture::write(bytes)

Efficiently upload raw pixel data into an existing texture. Ideal for video playback or any per-frame dynamic image updates.

- Whole texture updates: use `Texture.write(&bytes)` or `Texture.write_with(&bytes, TextureWriteOptions::whole())`.
- Sub-rectangle updates: pass origin and size via `TextureWriteOptions`.
- Bytes per row must be a multiple of 256. When unspecified, compute it from the pixel stride and align up.

## Notes
- Supported formats initially: `Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8Unorm`, `Bgra8UnormSrgb`, and other 4-bytes-per-pixel formats. Unsupported formats return an error.
- The texture must have COPY_DST usage.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture([64, 64], TextureFormat::Rgba, None).await?;
let frame_bytes = vec![0u8; 64 * 64 * 4];

texture.write(&frame_bytes)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
