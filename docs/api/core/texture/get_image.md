# Texture::get_image()

Read back the mip-0 contents of this texture as tightly-packed bytes in the texture's native format.

- Works on both native and WASM by awaiting the GPU readback buffer mapping via an async oneshot channel.
- Layer-major on the outside loop when the texture has multiple layers.
- The texture must have `COPY_SRC` usage; creation helpers like `create_storage_texture` enable it by default.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture(([64u32, 64u32], TextureFormat::Rgba)).await?;
texture.write(&vec![0u8; 64 * 64 * 4])?;

let bytes = texture.get_image().await?;
# assert_eq!(bytes.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
