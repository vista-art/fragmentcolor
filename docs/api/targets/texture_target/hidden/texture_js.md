# TextureTarget::texture() (JS/web)

Returns a [`Texture`](https://fragmentcolor.org/api/texture/texture) handle for the offscreen render target.

Use this to bind the texture as a shader uniform, for example when rendering a
post-processing pass that reads the output of a previous render step.

This is a web-specific binding because on WASM the texture handle is needed
for explicit uniform binding. On Rust/Python/mobile the `TextureTarget` is
passed directly to `renderer.render()` and the texture binding is handled
automatically.

## Example

```js
import { Renderer, Shader } from "fragmentcolor";

const renderer = new Renderer();

// Render some scene into an offscreen target, then read it back as a
// texture in a second fullscreen pass.
const offscreen = await renderer.createTextureTarget([512, 512]);
const output = await renderer.createTextureTarget([512, 512]);

const postShader = new Shader(`
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p  = array<vec2<f32>, 3>(vec2<f32>(-1., -1.), vec2<f32>(3., -1.), vec2<f32>(-1., 3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0., 1.), vec2<f32>(2., 1.), vec2<f32>(0., -1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}

@group(0) @binding(0) var t: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return textureSample(t, samp, in.uv);
}
`);

// `texture()` hands back the offscreen target's color texture so it can be
// bound as a shader uniform on web.
const tex = offscreen.texture();
await postShader.set("t", tex);
renderer.render(postShader, output);
```
