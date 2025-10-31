# TextureWriteOptions

Options to control how raw bytes are uploaded into a texture.

Fields
- origin_x, origin_y, origin_z
- size_width, size_height, size_depth
- bytes_per_row: optional; must be a multiple of 256 when provided
- rows_per_image: optional; defaults to height

## Example
```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::TextureWriteOptions;
let width = 64u32; let height = 64u32;
let pixel = 4u32; let stride = width * pixel;
let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
let bpr = ((stride + align - 1) / align) * align;
let _opt = fragmentcolor::TextureWriteOptions::whole()
  .with_bytes_per_row(bpr)
  .with_rows_per_image(height);
# Ok(())
# }
```
