# Texture.writeRegion(bytes, region)

JavaScript wrapper for `Texture::write_region`.

The `region` argument accepts:
- `[w, h]`: size only, origin `(0, 0, 0)`
- `[x, y, w, h]`: 2D rectangle
- `[x, y, z, w, h, d]`: 3D box
- `{ x, y, z?, width, height, depth?, bytesPerRow?, rowsPerImage? }`
- `{ minX, minY, maxX, maxY, minZ?, maxZ?, bytesPerRow?, rowsPerImage? }`

The origin keys `x`, `y`, `minX` and `minY` default to 0, and a missing depth or `maxZ` makes the region one layer deep. An object without its `width` and `height`, or its `maxX` and `maxY`, with a max at or below its min, or with keys from both forms, throws an `Error`, and so does a width, height or depth of 0 in any form. A missing region, `undefined` or `null`, writes the whole texture.

## Example

```js
import { Renderer, TextureFormat } from "fragmentcolor";

const renderer = new Renderer();
const texture = await renderer.createStorageTexture([64, 32], TextureFormat.Rgba);
const bytes = new Uint8Array(64 * 32 * 4);

// Simple sub-rectangle update.
texture.writeRegion(bytes, [0, 0, 64, 32]);

// Explicit data layout (advanced).
texture.writeRegion(bytes, { x: 0, y: 0, width: 64, height: 32, bytesPerRow: 256, rowsPerImage: 32 });
```
