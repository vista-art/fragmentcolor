# Mipmap::count

Return the number of mip levels in the chain (always `>= 1`). For a base size of `w x h`, this is `1 + floor(log2(max(w, h)))`.

## Example

```rust
use fragmentcolor::{Mipmap, TextureFormat};

let pixels: Vec<u8> = vec![0; 8 * 8 * 4];
let chain = Mipmap::build((
    pixels.as_slice(),
    TextureFormat::Rgba8UnormSrgb,
    [8, 8],
))?;
let count = chain.count();
# assert_eq!(count, 4); // 8, 4, 2, 1
let _ = count;
# Ok::<(), Box<dyn std::error::Error>>(())
```
