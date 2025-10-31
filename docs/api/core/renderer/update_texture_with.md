# Renderer::update_texture_with(id, bytes, options)

Same as `Renderer::update_texture`, but allows specifying origin, size, and layout.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat, TextureWriteOptions};
let renderer = Renderer::new();
let id = *renderer
    .create_storage_texture([640, 480], TextureFormat::Rgba, None)
    .await?
    .id();

let width = 640u32;
let height = 480u32;
let pixel = 4u32;
let stride = width * pixel;
let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
let bpr = ((stride + align - 1) / align) * align;
let required = (bpr * (height - 1) + stride) as usize;
let frame = vec![0u8; required];
let opt = TextureWriteOptions::whole().with_bytes_per_row(bpr);

renderer.update_texture_with(id, &frame, opt)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
``
