import { Renderer } from "fragmentcolor";

const renderer = new Renderer();
const element = document.getElementById("video");

if (element instanceof HTMLVideoElement) {
  const handle = renderer.createExternalTextureFromHtmlVideo(element);
  void handle;
}