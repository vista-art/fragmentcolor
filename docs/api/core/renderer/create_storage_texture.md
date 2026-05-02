# Renderer::create_storage_texture

Create a storage-class texture (for compute shaders, image store/load, or as a render target). Same `TextureInput` transport as `Renderer::create_texture` and `TextureMipChain::prepare` — one vocabulary across the API.

The `From<T>` impls cover the common shapes:

- `(size, format)` → empty storage texture, no initial data.
- `(size, format, bytes)` → storage texture pre-seeded with `bytes`.
- Explicit `TextureInput { data, options }` literal — full control (set `options.usage` for non-default usage flags via `TextureOptions::with_usage(...)`).

`options.size` is **required** for this entry — there's no source to infer dimensions from. Returns `TextureError::InvalidInput` when missing.

`options.usage` overrides the default `STORAGE | TEXTURE | COPY_SRC | COPY_DST` mask; `None` keeps the default. Cross-language bindings expose this as the underlying `u32` bit mask so it crosses every FFI cleanly.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat};

let r = Renderer::new();
// Empty storage texture — same single create_storage_texture entry.
let tex = r.create_storage_texture(([64u32, 64u32], TextureFormat::Rgba)).await?;
# _ = tex;

// Pre-seeded with bytes — same method, three-tuple form.
let pixels = vec![0u8; 64 * 64 * 4];
let tex2 = r
    .create_storage_texture(([64u32, 64u32], TextureFormat::Rgba, pixels))
    .await?;
# _ = tex2;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
