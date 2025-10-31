{
    use wasm_bindgen.JsCast
from fragmentcolor import Renderer
renderer = Renderer()
window = web_sys.window()
document = window.document()
element = document.get_element_by_id("video")
video = element.dyn_into.<web_sys.HtmlVideoElement>()
_handle = renderer.create_external_texture_from_html_video(video)
}
window = web_sys.window()
document = window.document()
element = document.get_element_by_id("video")
video = element.dyn_into.<web_sys.HtmlVideoElement>()

handle = renderer.create_external_texture_from_html_video(video)
# Build a bind group layout with externalTexture + sampler and bind `handle`.