# Texture.writeRegion(bytes, region)

JavaScript wrapper for `Texture::write_region`.

The `region` argument accepts:
- `[w, h]`: size only, origin `(0, 0, 0)`
- `[x, y, w, h]`: 2D rectangle
- `[x, y, z, w, h, d]`: 3D box
- `{ x, y, z?, width, height, depth?, bytesPerRow?, rowsPerImage? }`
- `{ minX, minY, maxX, maxY, minZ?, maxZ?, bytesPerRow?, rowsPerImage? }`

## Example

```js
import { Renderer, TextureFormat } from "fragmentcolor";

const renderer = new Renderer();
const texture = await renderer.createStorageTexture([64, 32], TextureFormat.Rgba, null);
const bytes = new Uint8Array(64 * 32 * 4);

// Simple sub-rectangle update.
texture.writeRegion(bytes, [0, 0, 64, 32]);

// Explicit data layout (advanced).
texture.writeRegion(bytes, { x: 0, y: 0, width: 64, height: 32, bytesPerRow: 256, rowsPerImage: 32 });
```
