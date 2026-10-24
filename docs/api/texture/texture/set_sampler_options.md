# Texture::set_sampler_options

Update the texture sampler options (filtering, wrapping, etc.).

`smooth: true` (the default) uses linear filtering for both magnification and minification, including trilinear interpolation between mip levels when the texture has a mipmap chain (created automatically for source images). `smooth: false` uses nearest-neighbor for everything.

The options apply from the next render to the `sampler` declared in the same bind group as the texture. When a group holds several textures, each sampler takes the options of the texture with the lowest `@binding` between the previous sampler and itself, or of the nearest texture if none sits in between. A single sampler after five textures uses the first texture's options.

A plain `sampler` ignores `compare`. A `sampler_comparison` uses `compare`, or `LessEqual` when it is `None`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Size, SamplerOptions};
let renderer = Renderer::new();
// 1x1 RGBA (white) raw pixel bytes
let pixels: &[u8] = &[255,255,255,255];

let texture = renderer.create_texture((pixels, [1, 1])).await?;
let opts = SamplerOptions { repeat_x: true, repeat_y: true, smooth: true, compare: None };
texture.set_sampler_options(opts);


# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
