# Mipmap::count

Return how many mip levels the chain holds (always at least 1). For a base size of `w x h`, that's `1 + floor(log2(max(w, h)))`. So an 8x8 source yields four levels (8, 4, 2, 1). Handy for asserting that the chain has the depth you expected before you upload, or for sizing a fixed-array view of the levels in your own code.

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
let count = chain.count();
# assert_eq!(count, 4); // 8, 4, 2, 1
# let _count = count;
# Ok::<(), Box<dyn std::error::Error>>(())
```
