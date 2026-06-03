# Scene::load

Build a `Scene` from a serialized 3D file. Pass a **path** (`.gltf` or `.glb`)
or in-memory **`.glb` bytes** straight to `Scene::load`. The format is
inferred from the input, so there's nothing extra to name.

`Scene::load` is synchronous and takes no `Renderer`. Any textures the parser
encounters are queued as pending uploads on the resulting Materials; the
renderer drains them on first render, or earlier if you call
[`Renderer::load`](https://fragmentcolor.org/api/core/renderer/load).

Today's coverage: static glTF, meaning mesh primitives (POSITION + NORMAL + UV0 +
indices), PBR-MR materials with all five texture slots, per-node transforms
flattened into Model matrices, glTF camera nodes, and `KHR_lights_punctual`
lights. Animation, skinning, morph targets, and material extensions beyond
PBR-MR are out of scope; they parse cleanly and the loader ignores them.

## Skipping embedded cameras and lights

By default the loader instantiates the glTF file's own camera and light nodes.
When you bring your own (a spring-arm rig, a UI-locked overlay camera, an
animated key/fill setup) the embedded ones would only fight for the same
shader uniforms. Load through `gltf(...)` to get a builder with `cameras(false)`
and `lights(false)` toggles:

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Scene, SceneSource};

// Load geometry + materials only; drop the file's camera and lights.
let scene = Scene::load(
    SceneSource::gltf("path/to/model.glb")
        .cameras(false)
        .lights(false),
)?;

// Supply your own camera; it's now the only one the Scene tracks.
let camera = Camera::perspective(1.047, 16.0 / 9.0, 0.1, 100.0)
    .look_at([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
scene.add(&camera)?;
# Ok(())
# }
```

## Threading and WASM

`Scene::load` is fully synchronous: the underlying importer decodes embedded
images and buffers on the calling thread. A 100 MB `.glb` freezes that thread
for the duration. Two implications:

- **Native**: spawn `Scene::load` on a worker thread (`std::thread::spawn`,
  `tokio::task::spawn_blocking`, …) if you can't afford a frame stall.
- **WASM**: a sync load on the main thread freezes the page, and the path form
  isn't available (the importer goes through `std::fs`). Fetch the `.glb` bytes
  via `fetch` inside a Web Worker, hand them to `Scene::load(bytes)`, then
  transfer the produced Scene back.

## Example

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Scene;

// A path — `.gltf` JSON (with external buffers/images) or a `.glb` container.
let scene = Scene::load("path/to/model.gltf")?;

// In-memory `.glb` bytes — fetched from disk, the network, or another
// asset pipeline before this point.
# let glb_bytes: Vec<u8> = std::fs::read("path/to/model.glb")?;
let scene2 = Scene::load(glb_bytes)?;
# let _ = (scene, scene2);
# Ok(())
# }
```
