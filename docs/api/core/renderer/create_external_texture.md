# Renderer::create_external_texture(source)

Wrap a native platform video-frame source as an external texture so a WGSL
shader can sample it directly via `texture_external` /
`textureSampleBaseClampToEdge`, without an intermediate CPU upload.

The `source` argument is platform-specific:

- **Web**: `HTMLVideoElement` (or anything that decodes into one).
- **iOS**: a `CVPixelBuffer`-backed handle (passed as a raw `UInt64` pointer
  over the uniffi boundary).
- **Android**: a `SurfaceTexture` handle (passed as a raw `ULong` pointer).

Currently returns an error on every platform. The public signature is
stable; native-side decode hookup is on the roadmap. Track support via
the typed `RendererError` you get back.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Renderer;
let renderer = Renderer::new();
// platform-specific source handle passed here
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
