# TextureMipChain::prepare (JavaScript)

JavaScript override for `TextureMipChain::prepare`. The JS binding takes
positional args `(bytes, format, size?)` with `size` optional. Pass a
real PNG or JPEG buffer for the encoded path; pass raw RGBA bytes plus
a `size` for the raw path.

## Example

```js
import { Renderer, TextureFormat, TextureMipChain } from "fragmentcolor";

// Encoded path - prepare from PNG / JPEG bytes (size is inferred).
// Use a fixture you have on hand. The favicon below is served by the
// healthcheck server so the example runs end-to-end without the test
// having to bring its own bytes.
const pngResp = await fetch("/healthcheck/public/favicon.png");
const pngBytes = new Uint8Array(await pngResp.arrayBuffer());
const chain = TextureMipChain.prepare(pngBytes, TextureFormat.Rgba8UnormSrgb);

// Raw pixel path: include the size so prepare skips decoding.
const rawRgba = new Uint8Array(8 * 8 * 4);
rawRgba.fill(200);
const chainRaw = TextureMipChain.prepare(rawRgba, TextureFormat.Rgba8UnormSrgb, [8, 8]);

// Upload the chain through the regular createTexture entry point.
const renderer = new Renderer();
const texture = await renderer.createTexture(chain);
const _ = chainRaw.levelCount();
const __ = texture.size();
```
