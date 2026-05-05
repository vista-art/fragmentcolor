# Renderer::create_external_texture(source)

Wrap a native platform video-frame source as an external texture so a WGSL
shader can sample it directly via `texture_external` /
`textureSampleBaseClampToEdge`, without an intermediate CPU upload.

The `source` argument is platform-specific:

- **Web**: `HTMLVideoElement` (or anything that decodes into one).
- **iOS**: a `CVPixelBuffer`-backed handle (passed as a raw pointer over the
  uniffi boundary).
- **Android**: a `SurfaceTexture` handle.

Returns `RendererError::Error("not implemented yet")` on every platform
today — the API surface is in place, the implementation is a follow-up.

## Example

```rust
# #[cfg(target_arch = "wasm32")]
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# use wasm_bindgen::JsCast;
use fragmentcolor::Renderer;
let renderer = Renderer::new();
# let video = web_sys::window().unwrap()
#     .document().unwrap()
#     .get_element_by_id("video").unwrap()
#     .dyn_into::<web_sys::HtmlVideoElement>().unwrap();
let handle = renderer.create_external_texture(&video);
let _ = handle;
# Ok(())
# }
# fn main() {}
```
