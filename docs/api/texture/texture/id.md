# Texture::id()

Return the stable `TextureId` for this texture. The id is valid within
the `Renderer` that created it and is the handle uniforms use to refer to
the texture. It names the texture while something holds it: a `Texture`
handle, the `TextureTarget` that owns it, or a shader uniform or pass target
that binds it. An id on its own doesn't keep the texture in the registry.

A `Material` map set from bytes, a path, a URL or a `Mipmap` keeps one id
in every `Renderer`, where it names the copy that `Renderer` loaded. The
copies go when the map does, so only the map's own slot binds that id, and
any other slot refuses it with `ShaderError::ForeignMap`. To share a map,
pass the same source to each Material.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture(([64, 64], TextureFormat::Rgba)).await?;
let id = *texture.id();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
