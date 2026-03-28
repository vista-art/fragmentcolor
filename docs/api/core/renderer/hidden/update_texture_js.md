# Renderer.updateTexture(id, bytes)

JavaScript wrapper for `Renderer::update_texture`.

## Example

```js
import { Renderer, TextureFormat } from "fragmentcolor";

const renderer = new Renderer();
const texture = await renderer.createStorageTexture([64, 64], TextureFormat.Rgba, null);
const id = texture.id();
const frame = new Uint8Array(64 * 64 * 4);

renderer.updateTexture(id, frame);
```
