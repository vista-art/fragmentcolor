# Texture.writeWith(bytes, options)

JavaScript wrapper for `Texture::write_with`.

## Example

```js
import { Renderer, TextureFormat, TextureWriteOptions } from "fragmentcolor";

const renderer = new Renderer();
const texture = await renderer.createStorageTexture([64, 32], TextureFormat.Rgba, null);
const frame = new Uint8Array(64 * 32 * 4);
const opt = TextureWriteOptions.whole().withBytesPerRow(256).withRowsPerImage(32);

texture.writeWith(frame, opt);
```
