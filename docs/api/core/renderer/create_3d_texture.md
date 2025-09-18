# Renderer::create_3d_texture

Create a 3D texture. This is a thin alias that forwards to `Renderer::create_texture_with`.

## Example

```rust
# use fragmentcolor::{Renderer, Size};
let r = Renderer::new();
let size = fragmentcolor::Size { width: 16, height: 16, depth: Some(8) };
let voxels = vec![0u8; (16*16*8*4) as usize];
let tex = futures::executor::block_on(
    r.create_3d_texture(&voxels, Some(size), Some(wgpu::TextureFormat::Rgba8Unorm))
).unwrap();
```
