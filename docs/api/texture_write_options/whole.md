# TextureWriteOptions::whole()

Create an options object that targets the entire texture region.

- Origin defaults to (0,0,0)
- Size is inferred from the texture at call time
- bytes_per_row and rows_per_image are inferred if not set

## Example
```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::TextureWriteOptions;
let _opt = fragmentcolor::TextureWriteOptions::whole();
# Ok(())
# }
```
