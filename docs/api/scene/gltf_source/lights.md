# GltfSource::lights

Toggle whether [`Scene::load`](https://fragmentcolor.org/api/scene/scene/load)
instantiates a [`Light`](https://fragmentcolor.org/api/scene/light) for
each glTF `KHR_lights_punctual` node it walks. Default: `true` — the
loader honours every directional / point / spot light shipped with the
asset.

Pass `false` when the consumer brings its own lighting rig (cursor
lighting, animated key/fill, dynamic emitters) and the glTF's embedded
lights would just consume slots in the 32-light cap.

Returns the same `GltfSource` for chaining with the matching
[`cameras`](https://fragmentcolor.org/api/scene/gltf_source/cameras) toggle.

## Example

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Light, Scene, SceneSource};

// Geometry only — supply our own lighting rig.
let scene = Scene::load(
    SceneSource::gltf("path/to/model.glb")
        .cameras(false)
        .lights(false),
)?;

scene.add(&Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]))?;
scene.add(&Light::point([2.0, 1.0, 0.0], [1.0, 0.4, 0.2]).set_intensity(4.0))?;

assert_eq!(
    scene.lights().len(),
    2,
    "filter dropped the glTF's lights; only our rig should be tracked"
);
# Ok(())
# }
```
