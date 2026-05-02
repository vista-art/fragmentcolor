# TextureMipChain::levels

Return the tightly-packed bytes for each mip level, level 0 first. Each level has `bytes_per_pixel(format) * max(1, base_w >> level) * max(1, base_h >> level)` bytes.

This is mostly useful for inspection, debugging, or persisting the chain to disk. The renderer reads the bytes directly when uploading via `Renderer::create_texture(chain)`; you do not need to copy them yourself.

## Example

```rust
use fragmentcolor::{TextureFormat, TextureMipChain};

let pixels: Vec<u8> = vec![0; 8 * 8 * 4];
let chain = TextureMipChain::prepare((
    pixels.as_slice(),
    TextureFormat::Rgba8UnormSrgb,
    [8u32, 8u32],
))?;
let level_zero_bytes = &chain.levels()[0];
# assert_eq!(level_zero_bytes.len(), 8 * 8 * 4);
let _ = level_zero_bytes;
# Ok::<(), Box<dyn std::error::Error>>(())
```
