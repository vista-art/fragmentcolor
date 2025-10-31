# Texture::write_with(bytes, options)

Same as `Texture::write`, but allows specifying origin, size, bytes_per_row and rows_per_image.

## Notes
- See `Texture::write` for format and alignment details.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat, TextureWriteOptions};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture([640, 480], TextureFormat::Rgba, None).await?;

// Upload a 320x240 region starting at (x=100, y=50)
let w = 320u32;
let h = 240u32;
let pixel = 4u32;
let stride = w * pixel;
let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
let bpr = ((stride + align - 1) / align) * align;
let required = (bpr * (h - 1) + stride) as usize;
let region_bytes = vec![0u8; required];
let opt = TextureWriteOptions {
    origin_x: 100,
    origin_y: 50,
    origin_z: 0,
    size_width: w,
    size_height: h,
    size_depth: 1,
    bytes_per_row: Some(bpr),
    rows_per_image: Some(h),
};
texture.write_with(&region_bytes, opt)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
``
