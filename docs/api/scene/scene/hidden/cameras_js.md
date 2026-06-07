# Scene.cameras (JS/web)

Web-specific example for [`Scene::cameras`](https://fragmentcolor.org/api/scene/scene/cameras).

Same behaviour as every other binding; the only difference is how the Scene
is loaded. On the web there's no filesystem, so the `.glb` bytes are fetched
and passed to `Scene.load` (see
[`Scene::load`](https://fragmentcolor.org/api/scene/scene/load)).

## Example

```js
import { Scene } from "fragmentcolor";

const response = await fetch("/healthcheck/public/model.glb");
const bytes = new Uint8Array(await response.arrayBuffer());
const scene = Scene.load(bytes);

// Animate every camera the glTF shipped per frame instead of supplying
// our own. Most scenes carry a single camera, so the loop body usually
// runs once.
for (const camera of scene.cameras()) {
  camera.lookAt([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
  camera.setAspect(16.0 / 9.0);
}
```
