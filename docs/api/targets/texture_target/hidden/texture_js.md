# TextureTarget::texture() — JS/web

Returns a [`Texture`](https://fragmentcolor.org/api/texture/texture) handle for the offscreen render target.

Use this to bind the texture as a shader uniform — for example when rendering a
post-processing pass that reads the output of a previous render step.

This is a web-specific binding because on WASM the texture handle is needed
for explicit uniform binding. On Rust/Python/mobile the `TextureTarget` is
passed directly to `renderer.render()` and the texture binding is handled
automatically.

## Example

```js
const renderer = new Renderer();
const target = await renderer.createTextureTarget([512, 512]);
const postShader = new Shader(`
  @group(0) @binding(0) var t: texture_2d<f32>;
  @fragment fn main() -> @location(0) vec4f {
    return textureSample(t, s, in.uv);
  }
`);
const tex = target.texture();
await postShader.set("t", tex);
renderer.render(postShader, target);
```
