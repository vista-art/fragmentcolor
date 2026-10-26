# Texture::get_image()

Read back the mip-0 contents of this texture as tightly-packed bytes in the texture's native format.

- Works on native and on the web; the call awaits the GPU readback mapping.
- For multi-layer textures, layers come first in the byte layout (layer-major).
- The texture must have `COPY_SRC` usage. Creation helpers like `create_storage_texture` enable it by default.
- GLES and WebGL2 can't copy a depth texture to a buffer, so reading one back there returns `TextureError::Readback`. So does a texture whose readback needs a buffer past the device's largest, `max_buffer_size`.
- A block-compressed texture, such as one loaded from a BC7 or ASTC KTX2 file, has no size per pixel, so reading it back is an `InvalidInput` error.
- Python returns the same bytes as a uint8 NumPy array shaped `(height, width, bytes per pixel)`, with a leading layer axis for array textures. A float format reads as floats through `image.view(numpy.float32)`.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture(([64, 64], TextureFormat::Rgba)).await?;
texture.write(&vec![0; 64 * 64 * 4])?;

let bytes = texture.get_image().await?;
# assert_eq!(bytes.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
