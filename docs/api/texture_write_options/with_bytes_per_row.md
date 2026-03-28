# TextureWriteOptions::with_bytes_per_row(bytes)

Set the bytes-per-row for the upload. Must be a multiple of 256.

## Example
```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::TextureWriteOptions;
let _opt = TextureWriteOptions::whole().with_bytes_per_row(256);
# Ok(())
# }
```
