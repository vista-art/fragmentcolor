# Renderer.createExternalTextureFromHtmlVideo(video)

JavaScript wrapper for `Renderer::create_external_texture_from_html_video`.

## Example

```js
import { Renderer } from "fragmentcolor";

const renderer = new Renderer();
const element = document.getElementById("video");

if (element instanceof HTMLVideoElement) {
  const handle = renderer.createExternalTextureFromHtmlVideo(element);
  void handle;
}
```
