# Texture::set_sampler_options (JavaScript)

JavaScript override for `Texture::set_sampler_options`. Uses the JS
`createTexture(input, options?)` signature instead of the Rust
`(bytes, [w, h])` tuple. The sampler options object accepts both
camelCase (`repeatX`) and snake_case (`repeat_x`) keys.

## Example

```js
import { Renderer } from "fragmentcolor";
const renderer = new Renderer();
// 1x1 RGBA (white) raw pixel bytes
const pixels = new Uint8Array([255, 255, 255, 255]);
const texture = await renderer.createTexture(pixels, { size: [1, 1] });
const opts = { repeatX: true, repeatY: true, smooth: true, compare: null };
texture.setSamplerOptions(opts);
```
