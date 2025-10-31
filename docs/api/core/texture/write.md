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
let texture = renderer.create_storage_texture([1280, 720], TextureFormat::Rgba, None).await?;

let width = 1280u32;
let height = 720u32;
let pixel_size = 4u32; // RGBA8
let stride = width * pixel_size;
let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
let bpr = ((stride + align - 1) / align) * align; // align to 256

let required = (bpr * (height - 1) + stride) as usize;
let frame_bytes = vec![0u8; required];

texture.write(&frame_bytes)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
