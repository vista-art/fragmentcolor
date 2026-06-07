# Scene.lights (JS/web)

Web-specific example for [`Scene::lights`](https://fragmentcolor.org/api/scene/scene/lights).

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

// Darken every loaded light to half intensity for a moody pass.
for (const light of scene.lights()) {
  const current = light.intensity();
  light.setIntensity(current * 0.5);
}
```
