# Renderer::create_storage_texture

Create a storage-class texture for compute shaders, image store/load, or as a render target. The input shapes mirror `Renderer::create_texture` and `Mipmap::build`.

Common forms:

- `(size, format)`: empty storage texture, no initial data.
- `(size, format, bytes)`: storage texture pre-seeded with `bytes`.
- A full `TextureInput { data, options }` literal: pass `options.usage` (via `TextureOptions::with_usage(...)`) for non-default usage flags.

`size` is required; storage textures have no source to infer dimensions from. Missing it returns `TextureError::InvalidInput`.

`options.usage` overrides the default mask of `STORAGE | TEXTURE | COPY_SRC | COPY_DST`. The bindings expose it as a `u32` bitmask, so the same flag values work in every language.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};

let r = Renderer::new();

// Empty storage texture.
let tex = r.create_storage_texture(([64, 64], TextureFormat::Rgba)).await?;
# _ = tex;

// Pre-seeded with bytes.
let pixels = vec![0; 64 * 64 * 4];
let tex2 = r
    .create_storage_texture(([64, 64], TextureFormat::Rgba, pixels))
    .await?;
# _ = tex2;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
