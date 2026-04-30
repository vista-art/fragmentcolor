# Renderer::create_storage_texture_with_data

Create a storage-class texture pre-seeded with CPU bytes. Convenience wrapper that combines [`Renderer::create_storage_texture`] with a one-shot [`Texture::write`].

- Default usage: `STORAGE_BINDING | TEXTURE_BINDING | COPY_SRC | COPY_DST`. If you pass a custom usage mask it must include `COPY_DST`.
- Bytes must be tightly packed in the texture's native format: `width * height * depth * bytes_per_pixel(format)`. No per-row padding is required — the upload path does not use the 256-byte alignment rule that `copy_buffer_to_texture` imposes.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};

let r = Renderer::new();
let seed = vec![0u8; 8 * 8 * 4];
let tex = r
    .create_storage_texture_with_data([8, 8], TextureFormat::Rgba, &seed, None)
    .await?;

# _ = tex;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
