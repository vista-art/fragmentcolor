# TextureTarget::texture

Return a sampleable [`Texture`](https://fragmentcolor.org/api/texture/texture)
handle that aliases the offscreen render target's storage. Useful for
multi-pass pipelines: render into the `TextureTarget`, then bind the
returned `Texture` as a shader uniform on the next pass to read it back.

The first call registers the underlying texture object with the renderer
and caches the resulting `TextureId`; subsequent calls return a fresh
`Texture` handle pointing at the same registered id (no duplicate
registration). The returned handle is `Clone` — pass it around freely.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader};

let renderer = Renderer::new();
let target = renderer.create_texture_target([256u32, 256u32]).await?;

// Bind the offscreen target's contents as a uniform on a downstream
// post-processing shader.
let post = Shader::new(r#"
    @group(0) @binding(0) var input_image : texture_2d<f32>;
    @group(0) @binding(1) var input_sampler : sampler;

    @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
        let p = array<vec2<f32>, 3>(vec2f(-1.0,-1.0), vec2f(3.0,-1.0), vec2f(-1.0,3.0));
        return vec4<f32>(p[i], 0.0, 1.0);
    }
    @fragment fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
        return textureSample(input_image, input_sampler, vec2<f32>(0.5, 0.5));
    }
"#)?;
post.set("input_image", &target.texture())?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
