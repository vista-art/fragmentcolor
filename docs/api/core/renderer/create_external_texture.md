# Renderer::create_external_texture(source)

Wrap a native platform video-frame source as an external texture so a WGSL
shader can sample it directly via `texture_external` /
`textureSampleBaseClampToEdge`, without an intermediate CPU upload.

The method exists on the Web, iOS and Android bindings. Desktop Rust and
Python have no native video-frame source to pass, so they don't have it.
The `source` argument is platform-specific:

- **Web**: `HTMLVideoElement` (or anything that decodes into one).
- **iOS**: a raw `UInt64` pointer to a `CVPixelBuffer`.
- **Android**: a raw `ULong` pointer to a `SurfaceTexture`.

Every call returns an error for now. The signature is stable, and the
native decode hookup is on the roadmap. Track support via the typed
`RendererError` you get back.

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
