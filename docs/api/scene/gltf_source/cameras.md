# GltfSource::cameras

Toggle whether [`Scene::load`](https://fragmentcolor.org/api/scene/scene/load)
instantiates a [`Camera`](https://fragmentcolor.org/api/scene/camera)
for each glTF `camera` node it walks. Default: `true` — the loader
honours the glTF file's framing.

Pass `false` when the consumer brings its own Camera (a spring-arm rig,
a UI-locked overlay camera, a per-frame-animated viewpoint) and the
glTF's embedded camera would only fight for the same `camera.view_proj`
and `camera.position` uniforms. With `cameras(false)`, the loader walks
camera nodes but produces nothing — your subsequent
`scene.add(&my_camera)` is the only Camera the Scene sees, and the
sticky `has_camera` default-injection path still fires correctly if you
also skip supplying one.

Returns the same `GltfSource` (consumes-and-returns) for chaining with
the matching [`lights`](https://fragmentcolor.org/api/scene/gltf_source/lights)
toggle and the in-memory bytes / path conversion.

## Example

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Scene, SceneSource};

// Drop the glTF's embedded camera; supply our own.
let scene = Scene::load(
    SceneSource::gltf("path/to/model.glb").cameras(false),
)?;

let camera = Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0)
    .look_at([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
scene.add(&camera)?;

assert_eq!(
    scene.cameras().len(),
    1,
    "filter dropped the glTF camera; only our camera should be tracked"
);
# Ok(())
# }
```
