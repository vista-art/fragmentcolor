# Scene::load

Build a `Scene` from a serialized 3D file. Pass a **path** (`.gltf` or `.glb`)
or in-memory **`.glb` bytes** straight to `Scene::load`. The format is
inferred from the input, so there's nothing extra to name.

`Scene::load` is synchronous and takes no `Renderer`. Each image stays as
the file holds it, and every texture slot that reads it becomes a map of
that one source on the resulting Materials.
[`Renderer::load`](https://fragmentcolor.org/api/core/renderer#load) decodes
and uploads a copy of each map into that `Renderer`, so the Scene draws with
every `Renderer` that loads it. On native targets the first render runs it
for you; on the web, await it in each `Renderer` before its first render.

The Materials keep each image encoded for as long as a slot holds its map:
an image inside a `.glb` or a data URI keeps its file bytes, and an image in
a file beside a `.gltf` keeps only its path, which each `Renderer` reads
again. A scene keeps about its file's size in RAM beside its copy on each
GPU. The Material page's Memory section lists what each kind of source keeps.

`Scene::load` reads each image's header, its format and its size, and fails
with `SceneLoadError::Invalid`, naming the image, for one it can't read. A
corrupt image body loads with the scene and fails at the first `load` into
a `Renderer`, with an error that names the image.

The loader reads static glTF: mesh primitives (`POSITION`, `NORMAL`,
`TEXCOORD_0`, `TEXCOORD_1`, `COLOR_0`, `TANGENT` and indices), PBR-MR
materials with all five texture slots, per-node transforms flattened into
Model matrices, glTF camera nodes, and `KHR_lights_punctual` lights.
Animation, skinning, morph targets, and material extensions beyond PBR-MR
and `KHR_texture_transform` are out of scope; they parse cleanly and the
loader ignores them.

Each texture slot samples the UV set its `texCoord` names, through its own
`KHR_texture_transform`. The extension's `texCoord`, when it has one, wins
over the texture's. A primitive without `TEXCOORD_1` takes its `TEXCOORD_0`
values as `UV1`, so a slot on set 1 samples the same UVs as set 0 there.
The PBR material samples two UV sets, so a texture that names `TEXCOORD_2`
or above returns `SceneLoadError::Invalid`, naming the slot, such as
`occlusionTexture`.

Each texture keeps its glTF sampler's filters and wrap modes, but the PBR
material samples all five slots with one sampler. That sampler takes the
options of the first texture in slot order, base color first, so the
other maps follow its wrap modes. A file whose base color texture clamps
while a tiled normal map repeats draws the normal map clamped, and the
edge texels smear across most of the surface.

Triangle strips and fans load as triangle lists. glTF also allows a few
things the loader can't take, and it returns `SceneLoadError::Invalid` for
them:

- a primitive drawn as points or lines
- an index accessor, or a mesh attribute the loader reads (`POSITION`,
  `NORMAL`, `TANGENT`, `TEXCOORD_0`, `TEXCOORD_1` and `COLOR_0`), with more
  elements than 32-bit indices reach. Attributes it doesn't read, such as
  `JOINTS_0` or `COLOR_1`, aren't checked. On the web the glTF parser
  refuses any count past 32 bits first, so there the file returns
  `SceneLoadError::Gltf`.
- a sparse `POSITION` or index accessor without a bufferView that spans more
  bytes than all of the file's buffers hold

The glTF parser also refuses any accessor with neither a bufferView nor
sparse values, which glTF reads as all zeros. A file with one returns
`SceneLoadError::Gltf`, even when the accessor belongs to an animation or
nothing at all.

A file that breaks the glTF rules returns the same error. That covers:

- an accessor with a count of 0, a type its semantic doesn't take, or a
  byte range outside its buffer view or buffer
- an attribute with a different element count than `POSITION`
- an index past the last vertex
- a node reached twice, through a cycle or a child with two parents
- a point or spot light with a negative `range`
- a camera whose `zfar` isn't past its `znear`

Each of these messages names the accessor, node, light or camera at fault.
When no single accessor is to blame, it names the mesh and the primitive.

A camera that `Camera::perspective`, `Camera::orthographic` or
`Camera::look_at` refuses returns `SceneLoadError::Camera`, which names the
camera, whether or not its values break a glTF rule. That covers a `yfov`
that isn't between 0 and π, a `znear` of 0 and a camera node scaled to
zero. Load with `cameras(false)` to skip the file's cameras.

A perspective camera without `zfar` asks for an infinite projection, and
the loader builds one, so nothing in front of the camera clips however far
away.

## Skipping embedded cameras and lights

By default the loader instantiates the glTF file's own camera and light nodes.
When you bring your own (a spring-arm rig, a UI-locked overlay camera, an
animated key/fill setup) the embedded ones would only fight for the same
shader uniforms. Load through `GltfSource::from(...)` to get a builder with
`cameras(false)` and `lights(false)` toggles:

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, GltfSource, Scene};

// Load geometry + materials only; drop the file's camera and lights.
let scene = Scene::load(
    GltfSource::from("path/to/model.glb")
        .cameras(false)
        .lights(false),
)?;

// Supply your own camera; it's now the only one the Scene tracks.
let camera = Camera::perspective(1.047, 16.0 / 9.0, 0.1, 100.0)?
    .look_at([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])?;
scene.add(&camera)?;
# Ok(())
# }
```

The other bindings take the same two switches with `Scene.load`. Both
default to `true`:

- **JS:** `Scene.load(bytes, { cameras: false, lights: false })`
- **Python:** `Scene.load(source, cameras=False, lights=False)`
- **Swift:** `try Scene.load(source, cameras: false, lights: false)`
- **Kotlin:** `Scene.load(source, cameras = false, lights = false)`

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

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let glb_bytes: Vec<u8> = std::fs::read("path/to/model.glb")?;
# /*[python]
# glb_bytes = open("path/to/model.glb", "rb").read()
# */
# /*[swift]
# import Foundation
# let glb_bytes = try Data(contentsOf: URL(fileURLWithPath: "path/to/model.glb"))
# */
# /*[kotlin]
# val glb_bytes = java.io.File("path/to/model.glb").readBytes()
# */
use fragmentcolor::Scene;

// A path to a `.gltf` file with its buffers and images, or to a `.glb` file.
let scene = Scene::load("path/to/model.gltf")?;

// The bytes of a `.glb` file, read from disk or the network.
let scene2 = Scene::load(glb_bytes)?;
# let _ = (scene, scene2);
# Ok(())
# }
```
