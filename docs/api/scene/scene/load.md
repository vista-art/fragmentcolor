# Scene::load

Build a `Scene` from a serialized 3D format. The input is a
[`SceneSource`](https://fragmentcolor.org/api/scene/scene_source) — a
format-tagged enum so the loader knows which parser to dispatch. Today the
only variant is `Gltf`, covering glTF 2.0 JSON and `.glb` binary
containers; future formats slot in as new variants without disturbing the
public method.

`Scene::load` is sync and takes no `Renderer`. Any textures the parser
encounters are queued as pending [`TextureInput`](https://fragmentcolor.org/api/texture/texture_input)
entries on the resulting Materials; the renderer drains them on first
render, or earlier if you call
[`Renderer::load`](https://fragmentcolor.org/api/core/renderer/load).

Today's coverage: static glTF — mesh primitives (POSITION + NORMAL + UV0
+ indices), PBR-MR materials with all five texture slots, per-node
transforms flattened into Model matrices, glTF camera nodes, and
`KHR_lights_punctual` lights. Animation, skinning, morph targets, and
material extensions beyond PBR-MR are out of scope; they parse cleanly
and the loader ignores them.

## Example

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Scene, SceneSource};

// File path — covers both `.gltf` JSON (with external buffers/images)
// and `.glb` binary containers.
let scene = Scene::load(SceneSource::gltf("path/to/model.gltf"))?;

// In-memory `.glb` bytes — fetched from disk, network, or a BIN chunk
// in another format.
let glb_bytes: Vec<u8> = vec![/* … */];
let scene2 = Scene::load(SceneSource::gltf(glb_bytes))?;
# let _ = (scene, scene2);
# Ok(())
# }
```
