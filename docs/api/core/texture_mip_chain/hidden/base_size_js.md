# TextureMipChain::base_size (JavaScript)

JavaScript override for `TextureMipChain::base_size`. The Rust example
destructures a `(u32, u32)` tuple via `let (width, height) = ...`, which
isn't valid JS. The JS binding returns a `Size` object with `.width` and
`.height`, so we read those instead.

## Example

```js
import { TextureFormat, TextureMipChain } from "fragmentcolor";

const pixels = new Uint8Array(16 * 16 * 4);
const chain = TextureMipChain.prepare(pixels, TextureFormat.Rgba8UnormSrgb, [16, 16]);
const sz = chain.baseSize();
const width = sz.width;
const height = sz.height;
```
