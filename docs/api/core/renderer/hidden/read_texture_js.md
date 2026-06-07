# Renderer::readTexture(textureId)

JavaScript/WASM wrapper for `Renderer::read_texture`. Returns a `Promise<Uint8Array>`. Await it to get the tightly-packed pixel bytes for the registered texture in its native format.

## Example

```js
import { Renderer, TextureFormat } from "fragmentcolor";
const renderer = new Renderer();
const texture = await renderer.createStorageTexture([64, 64], TextureFormat.Rgba);
texture.write(new Uint8Array(64 * 64 * 4));

const bytes = await renderer.readTexture(texture.id());
```
