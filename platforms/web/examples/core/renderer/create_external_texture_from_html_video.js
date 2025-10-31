{
    use wasm_bindgen.JsCast;
import { Renderer } from "fragmentcolor";
const renderer = new Renderer();
const window = web_sys.window().unwrap();
const document = window.document().unwrap();
const element = document.getElementById("video").unwrap();
const video = element.dynInto.<web_sys.HtmlVideoElement>().unwrap();
const _handle = renderer.createExternalTextureFromHtmlVideo(video);
};
const window = web_sys.window().unwrap();
const document = window.document().unwrap();
const element = document.getElementById("video").unwrap();
const video = element.dynInto.<web_sys.HtmlVideoElement>().unwrap();

const handle = renderer.createExternalTextureFromHtmlVideo(video);
// Build a bind group layout with externalTexture + sampler and bind `handle`.