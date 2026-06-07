# Mipmap::size (JavaScript)

JavaScript override for `Mipmap::size`. The Rust example
destructures a `(u32, u32)` tuple via `let (width, height) = ...`, which
isn't valid JS. The JS binding returns a `Size` object with `.width` and
`.height`, so we read those instead.

## Example

```js
import { TextureFormat, Mipmap } from "fragmentcolor";

const pixels = new Uint8Array(16 * 16 * 4);
const chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [16, 16]);
const sz = chain.size();
const width = sz.width;
const height = sz.height;
```
