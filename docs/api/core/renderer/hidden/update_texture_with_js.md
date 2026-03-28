# Renderer.updateTextureWith(id, bytes, options)

JavaScript wrapper for `Renderer::update_texture_with`.

## Example

```js
import { Renderer, TextureFormat, TextureWriteOptions } from "fragmentcolor";

const renderer = new Renderer();
const texture = await renderer.createStorageTexture([64, 32], TextureFormat.Rgba, null);
const id = texture.id();
const frame = new Uint8Array(64 * 32 * 4);
const opt = TextureWriteOptions.whole().withBytesPerRow(256).withRowsPerImage(32);

renderer.updateTextureWith(id, frame, opt);
```
