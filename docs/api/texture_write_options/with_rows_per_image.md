# TextureWriteOptions::with_rows_per_image(rows)

Set the number of rows per image for the upload (usually equals the height).

## Example
```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::TextureWriteOptions;
let height = 64u32;
let _opt = fragmentcolor::TextureWriteOptions::whole().with_rows_per_image(height);
# Ok(())
# }
```
