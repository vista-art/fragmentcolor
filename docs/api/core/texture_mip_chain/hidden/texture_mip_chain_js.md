# TextureMipChain (JavaScript)

JavaScript example for `TextureMipChain`. Use the `prepare(bytes, format, size?)`
positional-args entry, then hand the chain to `renderer.createTexture(chain)`
for the GPU upload.

## Example

```js
import { Renderer, TextureFormat, TextureMipChain } from "fragmentcolor";

const renderer = new Renderer();
// Real encoded PNG bytes: served by the healthcheck server so the example
// runs end-to-end without packaging its own fixture.
const pngResp = await fetch("/healthcheck/public/favicon.png");
const pngBytes = new Uint8Array(await pngResp.arrayBuffer());
const chain = TextureMipChain.prepare(pngBytes, TextureFormat.Rgba8UnormSrgb);

// Hand the chain to the unified create_texture entry - same vocabulary as
// every other texture path; From<TextureMipChain> selects the GPU-only
// upload internally.
const texture = await renderer.createTexture(chain);
const _ = texture.size();
```
