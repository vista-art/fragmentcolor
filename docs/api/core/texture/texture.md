# Texture

A GPU texture resource. Public API wrapper that holds a shareable reference to the internal TextureObject and a numeric handle used by uniforms.

- Construct via [Renderer::create_texture(input)](https://fragmentcolor.org/api/core/renderer/create_texture). `input` accepts bytes, `(bytes, [w, h])`, `(bytes, format)`, a path, a URL, a KTX2 container, or a prepared chain.
- Bind to a shader with `shader.set("key", &texture)`.
- The texture owns its sampler; tweak filtering and wrapping via `set_sampler_options`.

## How to use

Create a [Texture](https://fragmentcolor.org/api/core/texture) and set it on a [Shader](https://fragmentcolor.org/api/core/shader)

## Example

In your WGSL, declare a sampler in the same group as your texture

```wgsl
// must be in the same group
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
```

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Shader, Size};
let renderer = Renderer::new();
let shader = Shader::new(r#"
@group(0) @binding(0) var my_texture: texture_2d<f32>;
@group(0) @binding(1) var my_sampler: sampler;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
"#)?;

// 1x1 white pixel. Passing a size tells create_texture to read the bytes
// as raw pixels; the default format is Rgba (sRGB-aware).
let pixels: &[u8] = &[255, 255, 255, 255];
let texture = renderer.create_texture((pixels, [1, 1])).await?;

// Bind the texture to the uniform name declared in WGSL.
shader.set("my_texture", &texture)?;

# _ = shader;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
