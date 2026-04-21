{
    use wasm_bindgen.JsCast
import org.fragmentcolor.*
val renderer = Renderer()
val window = 
        web_sys.window().okOrElse(|| std.io.Error.other("window not available"))
val document = window
        .document()
        .okOrElse(|| std.io.Error.other("document not available"))
val element = document
        .getElementById("video")
        .okOrElse(|| std.io.Error.other("video element not found"))
val video = element
        .dynInto.<web_sys.HtmlVideoElement>()
        .mapErr(|_| std.io.Error.other("element is not a video"))
val handle = renderer.createExternalTextureFromHtmlVideo(video)
val _ = handle
}