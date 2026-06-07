# Scene.load (JS/web)

Web-specific example for [`Scene::load`](https://fragmentcolor.org/api/scene/scene/load).

On the web there's no filesystem, so the path form of `Scene.load` isn't
available. Fetch the `.glb` bytes yourself (over `fetch`, from an asset
pipeline, or a `File` input) and hand the resulting `Uint8Array` to
`Scene.load`.

## Example

```js
import { Scene } from "fragmentcolor";

// Fetch the `.glb` container as bytes, then load it. The same call accepts
// a path string on native; on web pass the bytes instead.
const response = await fetch("/healthcheck/public/model.glb");
const bytes = new Uint8Array(await response.arrayBuffer());
const scene = Scene.load(bytes);
```
