# Texture::size (JavaScript)

JavaScript override for `Texture::size`. Uses the JS
`createTexture(input, options?)` signature instead of the Rust
`(bytes, [w, h])` tuple.

## Example

```js
import { Renderer } from "fragmentcolor";
const renderer = new Renderer();
const pixels = new Uint8Array([255, 255, 255, 255]);
const tex = await renderer.createTexture(pixels, { size: [1, 1] });
const sz = tex.size();
```
