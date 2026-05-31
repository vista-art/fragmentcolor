# Scene::cameras

Return a snapshot of every [`Camera`](https://fragmentcolor.org/api/scene/camera)
added to this Scene via [`Scene::add`](https://fragmentcolor.org/api/scene/scene/add),
including Cameras the loader instantiated from glTF `camera` nodes
(unless you skipped them via the camera filter on
[`Scene::load`](https://fragmentcolor.org/api/scene/scene/load)).

Each entry is an Arc-shared clone of the original handle.
`camera.look_at(...)` / `camera.set_aspect(...)` on a returned handle
propagates the new view + projection to every shader the Camera is wired
into, the same live semantics as the Camera handle you originally added.

When the Scene rendered with a defaulted Camera (no user-supplied Camera
when the first render landed), the auto-injected default appears in
this list too, so a consumer who wants to drive the default camera per
frame can grab `scene.cameras().first()` and call `look_at` on it.

## Example

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Scene;

let scene = Scene::load("path/to/model.glb")?;

// glTF shipped a camera — animate it per frame instead of supplying our own.
if let Some(camera) = scene.cameras().into_iter().next() {
    camera.look_at([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    camera.set_aspect(16.0 / 9.0);
}
# Ok(())
# }
```
