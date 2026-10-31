# Mipmap::size

Return the base level (level 0) dimensions as a [Size](https://fragmentcolor.org/api/texture/texture#size) with no depth, in every language. Useful when you want to know what you're about to upload without keeping a separate copy of the source dimensions around. The chain has `1 + floor(log2(max(width, height)))` levels, each one half the previous size down to 1x1.

## Example

```rust
# let pixels = vec![0; 16 * 16 * 4];
# /*[js]
# const pixels = new Uint8Array(16 * 16 * 4);
# */
# /*[python]
# pixels = bytes(16 * 16 * 4)
# */
# /*[swift]
# import Foundation
# let pixels = Data(repeating: 0, count: 16 * 16 * 4)
# */
# /*[kotlin]
# val pixels = ByteArray(16 * 16 * 4)
# */
use fragmentcolor::{Mipmap, TextureFormat};

let chain = Mipmap::build((pixels.as_slice(), TextureFormat::Rgba8UnormSrgb, [16, 16]))?;
let size = chain.size();
let width = size.width;
let height = size.height;
# assert_eq!([width, height], [16, 16]);
# assert_eq!(size.depth, None);
# Ok::<(), Box<dyn std::error::Error>>(())
```
