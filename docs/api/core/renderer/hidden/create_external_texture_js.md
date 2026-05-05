# Renderer.createExternalTexture(video)

JavaScript wrapper for `Renderer::create_external_texture`. Accepts an
`HTMLVideoElement`.

## Example

```js
import { Renderer } from "fragmentcolor";

const renderer = new Renderer();
const element = document.getElementById("video");

if (element instanceof HTMLVideoElement) {
  const handle = renderer.createExternalTexture(element);
  void handle;
}
```
