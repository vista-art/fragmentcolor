# Texture::resize(size)

Gives the texture a new size, with the same format, usage and sample count.
Every clone of the handle follows, and so do the passes and shaders that hold it,
so a depth texture or an offscreen image can follow a window without being created again.
The pixels start blank at the new size. Asking for the current size keeps the texture as it is.

Only a texture a pass can draw into resizes: one from `create_depth_texture`, the `texture()` of a
`TextureTarget`, or a texture made with `RENDER_ATTACHMENT` in its usage by `create_storage_texture`
or `create_texture`. A texture made from an image loses that image when it resizes. Resizing a target's
`texture()` resizes the target. Any other texture returns `TextureError::FixedSize`, and a side of
zero, or one past the device limit, returns `TextureError::InvalidInput`. Both keep the old size.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Pass, Renderer};

let renderer = Renderer::new();
let depth = renderer.create_depth_texture([800, 600]).await?;
let pass = Pass::new("scene");
pass.set_depth_target(&depth)?;

// The window grew. The pass keeps the same depth texture at the new size.
depth.resize([1024, 768])?;

# let size: [u32; 2] = depth.size().into();
# assert_eq!(size, [1024, 768]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
