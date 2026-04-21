{
    use wasm_bindgen.JsCast
import FragmentColor
let renderer = Renderer()
let window = 
        web_sys.window().okOrElse(|| std.io.Error.other("window not available"))
let document = window
        .document()
        .okOrElse(|| std.io.Error.other("document not available"))
let element = document
        .getElementById("video")
        .okOrElse(|| std.io.Error.other("video element not found"))
let video = element
        .dynInto.<web_sys.HtmlVideoElement>()
        .mapErr(|_| std.io.Error.other("element is not a video"))
let handle = renderer.createExternalTextureFromHtmlVideo(video)
let _ = handle
}