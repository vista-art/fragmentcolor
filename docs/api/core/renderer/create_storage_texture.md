# Renderer::create_storage_texture

Create a storage-class texture for compute shaders, image store/load, or as a render target.

Common forms:

- `(size, format)`: empty storage texture, no initial data.
- `(size, format, bytes)`: storage texture pre-seeded with `bytes`.
- A full `TextureInput { data, options }` literal: pass `options.usage` (via `TextureOptions::with_usage(...)`) for non-default usage flags.

`size` is required; storage textures have no source to infer dimensions from. Missing it returns `RendererError::CreateTextureError`.

A depth of 1, or none, makes a 2D texture, and a larger depth makes a 3D texture. A side of zero, or a size past the device's limits for that kind of texture, is an error that reaches no GPU call: `TextureError::InvalidInput` in Rust, a thrown `Error` in JavaScript, `InvalidInputError` (a `FragmentColorError`) in Python, `FragmentColorError` in Swift and `FragmentColorException` in Kotlin.

`options.usage` overrides the default mask of `STORAGE_BINDING | TEXTURE_BINDING | COPY_SRC | COPY_DST`. Every binding has the same named `TextureUsage` flags, combined with `|` (`or` in Kotlin). The other bindings take the size, format, optional bytes and optional usage as separate arguments.

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
