# Mipmap::levels (JavaScript)

JavaScript override for `Mipmap::levels`. The JS binding
exposes a `level(index)` accessor returning a `Uint8Array`, rather than
a `levels()` collection.

## Example

```js
import { TextureFormat, Mipmap } from "fragmentcolor";

const pixels = new Uint8Array(8 * 8 * 4);
const chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [8, 8]);
const levelZeroBytes = chain.level(0);
const _ = levelZeroBytes;
```
