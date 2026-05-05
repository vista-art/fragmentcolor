# TextureMipChain::levels (JavaScript)

JavaScript override for `TextureMipChain::levels`. The JS binding
exposes a `level(index)` accessor returning a `Uint8Array`, rather than
a `levels()` collection.

## Example

```js
import { TextureFormat, TextureMipChain } from "fragmentcolor";

const pixels = new Uint8Array(8 * 8 * 4);
const chain = TextureMipChain.prepare(pixels, TextureFormat.Rgba8UnormSrgb, [8, 8]);
const levelZeroBytes = chain.level(0);
const _ = levelZeroBytes;
```
