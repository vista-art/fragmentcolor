# Scene::new

Build an empty `Scene`. No `Renderer` argument, no async, nothing to await
— the Scene's GPU resources initialise lazily when it's first rendered.

The first call to [`Scene::add`](https://fragmentcolor.org/api/scene/scene/add)
creates a default [Pass](https://fragmentcolor.org/api/core/pass) under the
hood and routes the object into it. Multiple
[SceneObjects](https://fragmentcolor.org/api/scene) share that one Pass
unless you explicitly add more via
[`Scene::add_pass`](https://fragmentcolor.org/api/scene/scene/add_pass).

`Scene` is `Clone` (shallow Arc-share) — cloning gives another handle to
the same underlying scene, so mutations on one clone are visible on the
other.

## Example

```rust
use fragmentcolor::Scene;

let scene = Scene::new();
// scene is empty; add Models / Cameras / Lights with `scene.add(...)`.
# let _ = scene;
```
