# Renderer::read_texture(texture_id)

Read back the mip-0 contents of a registered texture as tightly-packed bytes in the texture's native format.

- Works on native and on the web; the call awaits the GPU readback mapping.
- Equivalent to calling [`Texture::get_image`] on the texture handle. Use this entry point when you only kept the `TextureId` around.
- The texture must have `COPY_SRC` usage. Creation helpers like `create_storage_texture` enable it by default.
- GLES and WebGL2 can't copy a depth texture to a buffer, so reading one back there returns `TextureError::Readback`. So does a texture whose readback needs a buffer past the device's largest, `max_buffer_size`.
- A block-compressed texture, such as one loaded from a BC7 or ASTC KTX2 file, has no size per pixel, so reading it back is an `InvalidInput` error.
- In JavaScript, `readTexture` first waits for the frames rendered before it to report their GPU errors and throws the first one, as [`Renderer::render`](https://fragmentcolor.org/api/core/renderer#render) describes.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture(([64, 64], TextureFormat::Rgba)).await?;
texture.write(&vec![0; 64 * 64 * 4])?;

let bytes = renderer.read_texture(texture.id()).await?;
# assert_eq!(bytes.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
