# TextureWriteOptions::with_bytes_per_row(bytes)

Set the bytes-per-row for the upload. Must be a multiple of 256.

## Example
```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::TextureWriteOptions;
let width = 64u32; let pixel = 4u32; let stride = width * pixel;
let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
let bpr = ((stride + align - 1) / align) * align;
let _opt = fragmentcolor::TextureWriteOptions::whole().with_bytes_per_row(bpr);
# Ok(())
# }
```
