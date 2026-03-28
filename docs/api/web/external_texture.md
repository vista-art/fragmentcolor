# Web External Texture (video)

Bind an HTML video element as an external texture on Web. In WGSL, declare `texture_external` and sample with `textureSampleBaseClampToEdge`.

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
let window = web_sys::window().unwrap();
let document = window.document().unwrap();
let element = document.get_element_by_id("video").unwrap();
let video = element.dyn_into::<web_sys::HtmlVideoElement>().unwrap();

let handle = renderer.create_external_texture_from_html_video(&video)?;
// Build a bind group layout with externalTexture + sampler, then create a bind group using `handle`.
```

Notes
- Requires browser support for external textures.
- If not available, consider a fallback: draw the <video> into a canvas and upload pixels via `Texture.write_with` each frame.