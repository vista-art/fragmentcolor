# Mipmap::level

Return the bytes of one mip level, with level 0 the full-size image. An index past the last level returns `TextureError::InvalidInput`.

## Example

```rust
use fragmentcolor::{Mipmap, TextureFormat};

let pixels: Vec<u8> = vec![0; 8 * 8 * 4];
let chain = Mipmap::build((
    pixels.as_slice(),
    TextureFormat::Rgba8UnormSrgb,
    [8, 8],
))?;
let level_zero_bytes = chain.level(0)?;
# assert_eq!(level_zero_bytes.len(), 8 * 8 * 4);
# assert!(chain.level(9).is_err());
# Ok::<(), Box<dyn std::error::Error>>(())
```
