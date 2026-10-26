# Texture::write(bytes)

Efficiently upload raw pixel data into an existing texture, such as each frame of a video or any image that changes from frame to frame.

A texture from `create_texture` has a mip chain by default, and every whole write rebuilds that chain on the CPU, on the calling thread. For video and other per-frame updates, create the texture with `mipmaps: false`, so a write is a single GPU upload.

- Whole texture updates: use `Texture.write(&bytes)`.
- Sub-rectangle updates or explicit data layout: use `Texture.write_region(&bytes, region)`.
- `bytes` holds tightly packed rows: each row is the texture width times the bytes per pixel, with no padding. Data sized exactly for rows padded to 256 bytes reads at that padded stride. For any other padding, pass the stride to `Texture.write_region`. Data of any other length is an `InvalidInput` error, so a frame of the wrong size never uploads cropped.

## Notes
- Any format with a fixed size per pixel works, from 1-byte `R8Unorm` to 16-byte `Rgba32Float`. A block-compressed texture, such as one loaded from a BC7 or ASTC KTX2 file, is an `InvalidInput` error.
- The texture must have COPY_DST usage.
- `create_texture` builds a mip chain by default, and a whole write rebuilds it, so the texture looks the same up close and far away.
- A depth texture such as `Depth32Float` takes no uploaded pixels, so writing into one is an `InvalidInput` error. Draw into it instead.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture(([64, 64], TextureFormat::Rgba)).await?;
let frame_bytes = vec![0; 64 * 64 * 4];

texture.write(&frame_bytes)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
