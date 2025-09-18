# Renderer::create_2d_texture

Create a 2D texture. This is a thin alias that forwards to `Renderer::create_texture_with`.

## Example

```rust
# use fragmentcolor::{Renderer, Size};
let r = Renderer::new();
let size = Size::from((256, 256));
let pixels = vec![255u8; (256*256*4) as usize];
let tex = futures::executor::block_on(
    r.create_2d_texture(&pixels, Some(size), Some(wgpu::TextureFormat::Rgba8UnormSrgb))
).unwrap();
```
