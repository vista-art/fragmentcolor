# Renderer::read_texture(texture_id)

Read back the mip-0 contents of a registered texture as tightly-packed bytes in the texture's native format.

- Works on native and on the web; the call awaits the GPU readback mapping.
- Equivalent to calling [`Texture::get_image`] on the texture handle. Use this entry point when you only kept the `TextureId` around.
- The texture must have `COPY_SRC` usage. Creation helpers like `create_storage_texture` enable it by default.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture(([64, 64], TextureFormat::Rgba)).await?;
texture.write(&vec![0; 64 * 64 * 4])?;

let bytes = renderer.read_texture(*texture.id()).await?;
# assert_eq!(bytes.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
