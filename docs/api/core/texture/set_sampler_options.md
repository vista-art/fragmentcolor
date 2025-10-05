# Texture::set_sampler_options

Update the texture sampler options (filtering, wrapping, etc.).

Note: changes take effect on next bind; the renderer recreates the sampler as needed.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Size, SamplerOptions};
let renderer = Renderer::new();
// 1x1 RGBA (white) raw pixel bytes
let pixels: &[u8] = &[255,255,255,255];

let texture = renderer.create_texture_with_size(pixels, [1,1]).await?;
let opts = SamplerOptions { repeat_x: true, repeat_y: true, smooth: true, compare: None };
texture.set_sampler_options(opts);


# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
