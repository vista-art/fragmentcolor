# Mipmap::format (JavaScript)

JavaScript override for `Mipmap::format`.

## Example

```js
import { TextureFormat, Mipmap } from "fragmentcolor";

const pixels = new Uint8Array(4 * 4 * 4);
pixels.fill(200);
const chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [4, 4]);
const _ = chain.format();
```
