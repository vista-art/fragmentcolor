# Renderer::create_external_texture_from_html_video(video)

Create an external texture from an `HtmlVideoElement` for sampling in WGSL via `texture_external` and `textureSampleBaseClampToEdge`.

## Example
```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
#[cfg(target_arch = "wasm32")]
{
    use wasm_bindgen::JsCast;
    use fragmentcolor::Renderer;
    let renderer = Renderer::new();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let element = document.get_element_by_id("video").unwrap();
    let video = element.dyn_into::<web_sys::HtmlVideoElement>().unwrap();
    let _handle = renderer.create_external_texture_from_html_video(&video)?;
}
# Ok(())
# }
let window = web_sys::window().unwrap();
let document = window.document().unwrap();
let element = document.get_element_by_id("video").unwrap();
let video = element.dyn_into::<web_sys::HtmlVideoElement>().unwrap();

let handle = renderer.create_external_texture_from_html_video(&video)?;
// Build a bind group layout with externalTexture + sampler and bind `handle`.
```
