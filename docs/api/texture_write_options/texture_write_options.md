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
let _opt = TextureWriteOptions::whole().with_bytes_per_row(256).with_rows_per_image(64);
# Ok(())
# }
```
