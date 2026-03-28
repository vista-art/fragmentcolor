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
    let window =
        web_sys::window().ok_or_else(|| std::io::Error::other("window not available"))?;
    let document = window
        .document()
        .ok_or_else(|| std::io::Error::other("document not available"))?;
    let element = document
        .get_element_by_id("video")
        .ok_or_else(|| std::io::Error::other("video element not found"))?;
    let video = element
        .dyn_into::<web_sys::HtmlVideoElement>()
        .map_err(|_| std::io::Error::other("element is not a video"))?;
    let handle = renderer.create_external_texture_from_html_video(&video)?;
    let _ = handle;
}
# Ok(())
# }
```
