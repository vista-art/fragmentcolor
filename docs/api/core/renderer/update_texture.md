# Renderer::update_texture(id, bytes)

Update an existing texture by id using raw pixel bytes. This forwards to the texture’s write API under the hood.

- Use `update_texture(id, bytes)` for full-frame writes with sensible defaults.
- Use `update_texture_with(id, bytes, options)` to control origin, size, and layout.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
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

renderer.update_texture(id, &frame)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
