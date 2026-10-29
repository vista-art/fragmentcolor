# Pass::set_target(target)

Attach a per-pass color render target. When set, this pass renders into the provided texture instead of the final Target.

A Pass has at most one color target. Call `set_target` again to swap it. Use this to render intermediate results (e.g., a shadow map) that later passes can sample.

The texture needs `RENDER_ATTACHMENT` usage, a color format, one mip level and one layer. A pass draws into one level of one layer, so a texture with a mip chain or with layers returns `PassError::InvalidColorTarget`, and so does a depth format. `create_texture` builds a mip chain unless `mipmaps` is false. A texture from another `Renderer`, or one that `unregister_texture` removed, makes `render` return `RendererError::TextureNotFoundError`.

`render` checks the pass against its target before it takes a frame:

- Each shader writes at `@location(0)` the values the format takes: the same scalar type, in at least as many channels. An `Rgba32Uint` target takes `vec4<u32>`, so a shader that writes `vec4<f32>` returns `PassError::InvalidColorTarget`.
- A shader with `AlphaMode::Blend` needs a format the device blends. The integer formats never blend and return the same error. `Rgba32Float` blends only on a device with float32 blending, and on any other one returns `RendererError::FeatureError`, which Python raises as `FeatureError` and Swift and Kotlin throw as `Feature`. Blending reads the alpha channel, so the shader writes a `vec4`, whatever the format holds. The device's own blending is the last check of a pass, so a pass that fails another check, such as a format that never blends or an output without alpha, gets the same error on every device.
- The pass can't bind the texture it draws into. A shader that samples it returns `PassError::TextureConflict`. Sample it in a later pass.

A shader that writes nothing at `@location(0)`, such as one that only tests depth, leaves the target's color as it is.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Pass, TextureFormat};

let r = Renderer::new();
let tex_target = r.create_texture_target([512, 512]).await?;

let p = Pass::new("shadow");
p.set_target(&tex_target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
