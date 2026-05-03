# Renderer::wait_idle()

JavaScript wrapper for `Renderer::wait_idle`. No-op on WASM — the browser drives
submission readiness. Provided for API parity; web callers that need a sync point
should await a readback instead.

## Example

```js
import { Renderer, Shader } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget({ width: 8, height: 8 });
const shader = new Shader("void main() { fragColor = vec4(1.0); }");
renderer.render(shader, target);
renderer.waitIdle();
const bytes = target.getImage();
```
