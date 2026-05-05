# Web External Texture (video)

Bind an `HTMLVideoElement` as an external texture so shaders can sample decoded video frames directly via `texture_external` and `textureSampleBaseClampToEdge`, without copying pixels through the CPU.

> **Status:** the API surface is in place on every binding so portable code can be written against it today, but every entry point currently returns `"not implemented yet"`. Track [Renderer::create_external_texture](https://fragmentcolor.org/api/core/renderer/create_external_texture) for progress.

WGSL
```wgsl
@group(0) @binding(0) var samp: sampler;
@group(0) @binding(1) var video_tex: texture_external;

@fragment
fn fs_main(v_uv: vec2f) -> @location(0) vec4f {
  return textureSampleBaseClampToEdge(video_tex, samp, v_uv);
}
```

Rust (wasm)
```rust
# #[cfg(target_arch = "wasm32")]
# fn run() -> Result<(), Box<dyn std::error::Error>> {
# use wasm_bindgen::JsCast;
use fragmentcolor::Renderer;
let renderer = Renderer::new();
# let video = web_sys::window().unwrap()
#     .document().unwrap()
#     .get_element_by_id("video").unwrap()
#     .dyn_into::<web_sys::HtmlVideoElement>().unwrap();
let handle = renderer.create_external_texture(&video)?;
let _ = handle;
# Ok(())
# }
# fn main() {}
```

Today's workaround: draw the `<video>` into a canvas and upload pixels via `Texture.write_region` each frame.
