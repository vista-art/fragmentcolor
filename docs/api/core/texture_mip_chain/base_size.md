# TextureMipChain::base_size

Return the base level (level 0) dimensions as `(width, height)`. The mip chain has a level for each `1 + floor(log2(max(width, height)))`.

## Example

```rust
use fragmentcolor::{TextureFormat, TextureMipChain};

let pixels: Vec<u8> = vec![0; 16 * 16 * 4];
let chain = TextureMipChain::prepare((
    pixels.as_slice(),
    TextureFormat::Rgba8UnormSrgb,
    [16u32, 16u32],
))?;
let (width, height) = chain.base_size();
# assert_eq!(width, 16);
# assert_eq!(height, 16);
let _ = (width, height);
# Ok::<(), Box<dyn std::error::Error>>(())
```
