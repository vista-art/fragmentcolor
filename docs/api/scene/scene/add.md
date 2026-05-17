# Scene::add

Absorb any [`SceneObject`](https://fragmentcolor.org/api/scene) — a
[Model](https://fragmentcolor.org/api/scene/model),
[Camera](https://fragmentcolor.org/api/scene/camera),
[Light](https://fragmentcolor.org/api/scene/light), or a user-defined
node that implements the trait — into the scene. The Scene routes it
onto its default [Pass](https://fragmentcolor.org/api/core/pass), which
is created lazily on the first call.

Each kind brings its own attach behaviour: a Model queues a draw with its
own per-instance transform; a Camera or Light wires its uniforms into
every shader the pass renders, both the ones already there and the ones
added afterwards. Camera and Light hold Arc-shared state, so subsequent
mutations (`camera.look_at(...)`, `light.set_color(...)`) propagate to
every shader on the pass with no further `add` call.

Returns `Result<&Scene, PassError>` so Models can fail at attach time when
the Mesh layout doesn't match the Material's shader; Cameras and Lights
always succeed. Chain with `?` between calls.

If you add Models without adding a Camera or any Light, the Scene injects
sensible defaults at render time (`Camera::perspective` looking from
`[0, 0, 5]`, a white `Light::directional` pointing slightly off-axis) so
the first frame renders something recognisable. Add your own to take over.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Light, Material, Mesh, Model, Renderer, Scene, Vertex};

let renderer = Renderer::new();

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let model = Model::new(mesh, Material::pbr()?);

let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
    .look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);

let scene = Scene::new();
scene
    .add(&model)?
    .add(&camera)?
    .add(&sun)?;

// Updating the camera later is enough — every shader on the scene picks
// the new view_proj up at the next render.
camera.look_at([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
# let _ = renderer;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
