# Renderer::update_texture(id, bytes)

Update an existing texture by id using raw pixel bytes. This forwards to the texture’s write API under the hood.

- Use `update_texture(id, bytes)` for full-frame writes with sensible defaults.
- Use `update_texture_with(id, bytes, options)` to control origin, size, and layout.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture([64, 64], TextureFormat::Rgba, None).await?;
let id = *texture.id();
let frame = vec![0u8; 64 * 64 * 4];

renderer.update_texture(id, &frame)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
