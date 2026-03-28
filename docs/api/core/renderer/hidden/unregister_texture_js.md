# Renderer.unregisterTexture(id)

JavaScript wrapper for `Renderer::unregister_texture`.

## Example

```js
import { Renderer, TextureFormat } from "fragmentcolor";

const renderer = new Renderer();
const texture = await renderer.createStorageTexture([16, 16], TextureFormat.Rgba, null);
const id = texture.id();

renderer.unregisterTexture(id);
```
