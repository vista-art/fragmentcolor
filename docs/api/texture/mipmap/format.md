# Mipmap::format

Return the texture format the chain was built for: the `format` you passed to `build(...)`, with an alias such as `Rgba` returned as the concrete format it stands for, `Rgba8Unorm`. Useful when you receive a chain you didn't build yourself (passed in from another thread or stored in a cache) and need to confirm it matches the format your shader expects before uploading.

## Example

```rust
# let pixels: Vec<u8> = vec![200; 4 * 4 * 4];
# /*[js]
# const pixels = new Uint8Array(4 * 4 * 4).fill(200);
# */
# /*[python]
# pixels = bytes([200] * (4 * 4 * 4))
# */
# /*[swift]
# import Foundation
# let pixels = Data(repeating: 200, count: 4 * 4 * 4)
# */
# /*[kotlin]
# val pixels = ByteArray(4 * 4 * 4) { 200.toByte() }
# */
use fragmentcolor::{Mipmap, TextureFormat};

let chain = Mipmap::build((pixels.as_slice(), TextureFormat::Rgba8UnormSrgb, [4, 4]))?;
let format = chain.format();
# let _format = format;
# Ok::<(), Box<dyn std::error::Error>>(())
```
