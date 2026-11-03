# Mipmap::levels

Return the tightly-packed bytes for each mip level, level 0 first. Each level holds the format's bytes per pixel times `max(1, base_w >> level) * max(1, base_h >> level)` bytes. Reach for this when you want to inspect a level, dump the chain to disk, or feed the bytes into a tool that isn't FragmentColor. When you upload via `Renderer::create_texture(chain)`, the renderer reads the bytes directly and you don't need to touch them yourself.

## Example

```rust
# let pixels: Vec<u8> = vec![0; 8 * 8 * 4];
# /*[js]
# const pixels = new Uint8Array(8 * 8 * 4);
# */
# /*[python]
# pixels = bytes(8 * 8 * 4)
# */
# /*[swift]
# import Foundation
# let pixels = Data(repeating: 0, count: 8 * 8 * 4)
# */
# /*[kotlin]
# val pixels = ByteArray(8 * 8 * 4)
# */
use fragmentcolor::{Mipmap, TextureFormat};

let chain = Mipmap::build((pixels.as_slice(), TextureFormat::Rgba8UnormSrgb, [8, 8]))?;
let level_zero_bytes = &chain.levels()[0];
# assert_eq!(level_zero_bytes.len(), 8 * 8 * 4);
# let _level_zero_bytes = level_zero_bytes;
# Ok::<(), Box<dyn std::error::Error>>(())
```
