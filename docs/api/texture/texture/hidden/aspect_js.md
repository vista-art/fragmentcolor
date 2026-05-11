# Texture::aspect (JavaScript)

JavaScript override for `Texture::aspect`. Same shape as the Rust example,
just uses the JS `createTexture(input, options?)` signature instead of the
Rust `(bytes, [w, h])` tuple.

## Example

```js
import { Renderer } from "fragmentcolor";

const renderer = new Renderer();
// 1x1 RGBA (white) raw pixel bytes
const pixels = new Uint8Array([255, 255, 255, 255]);
const tex = await renderer.createTexture(pixels, { size: [1, 1] });
const a = tex.aspect();
```
