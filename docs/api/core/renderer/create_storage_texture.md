# Renderer::create_storage_texture

Create a storage-class texture for compute shaders, image store/load, or as a render target.

Common forms:

- `(size, format)`: empty storage texture, no initial data.
- `(size, format, bytes)`: storage texture pre-seeded with `bytes`, exactly the size's pixel count times the format's bytes per pixel.
- A full `TextureInput { data, options }` literal: pass `options.usage` (via `TextureOptions::with_usage(...)`) for non-default usage flags.

`size` is required; storage textures have no source to infer dimensions from. Missing it returns `RendererError::CreateTextureError`.

A depth of 1, or none, makes a 2D texture, and a larger depth makes a 3D texture. A side of zero, or a size past the device's limits for that kind of texture, is an error that reaches no GPU call: `TextureError::InvalidInput` inside `RendererError::TextureError` in Rust, a thrown `Error` named `InvalidInputError` in JavaScript, `InvalidInputError` (a `FragmentColorError`) in Python, `FragmentColorError` in Swift and `FragmentColorException` in Kotlin.

`options.usage` overrides the default mask of `STORAGE_BINDING | TEXTURE_BINDING | COPY_SRC | COPY_DST`. Every binding has the same named `TextureUsage` flags, combined with `|` (`or` in Kotlin). The other bindings take the size, format, optional bytes and optional usage as separate arguments. An empty mask, or a bit outside the five `TextureUsage` flags, is the same `InvalidInput` error. So is a usage the format can't take on this device, such as `STORAGE_BINDING` on `Depth32Float` or `Rgba8UnormSrgb`, and a depth texture with a depth past 1.

`R32Uint`, `R32Sint` and `R32Float` hold one 32-bit value per texel, which reads back as 4 little-endian bytes. A compute shader reads and writes them in place through a `read_write` storage slot, which WebGPU guarantees for these three formats, so it works in the browser too. An `R32Uint` or `R32Sint` texture also binds to an `atomic` slot, such as `texture_storage_2d<r32uint, atomic>`, where [texture atomics](https://fragmentcolor.org/api/core/shader#texture-atomics) change a texel safely from many invocations at once. They need no usage flag of their own. Texture atomics run on Vulkan, DX12, and Metal from macOS 14 and iOS 17. The browser's WebGPU and GL have none, so there `Renderer::render` returns `RendererError::FeatureError` for such a shader before it takes a frame.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader, TextureFormat};

let r = Renderer::new();

// Empty storage texture.
let tex = r.create_storage_texture(([64, 64], TextureFormat::Rgba)).await?;

// Pre-seeded with bytes.
let pixels = vec![0; 64 * 64 * 4];
let tex2 = r
    .create_storage_texture(([64, 64], TextureFormat::Rgba, pixels))
    .await?;

// One 32-bit counter per texel, for texture atomics.
let counts = r.create_storage_texture(([4, 4], TextureFormat::R32Uint)).await?;
let shader = Shader::new(r#"
@group(0) @binding(0) var counts: texture_storage_2d<r32uint, atomic>;

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
  textureAtomicAdd(counts, id.xy / 16u, 1u);
}
"#)?;
shader.set("counts", &counts)?;
# _ = (tex, tex2);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
