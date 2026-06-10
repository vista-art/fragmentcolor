# Scene

A `Scene` is the top-level container for the real-world things you render:
[Models](https://fragmentcolor.org/api/scene/model),
[Cameras](https://fragmentcolor.org/api/scene/camera),
[Lights](https://fragmentcolor.org/api/scene/light) (directional, point, and
spot are all one type), and any custom
[SceneObject](https://fragmentcolor.org/api/scene). It owns one or more
[Passes](https://fragmentcolor.org/api/core/pass) underneath and implements
[Renderable](https://fragmentcolor.org/api/core/renderable), so you hand
the whole scene to the [Renderer](https://fragmentcolor.org/api/core/renderer)
in a single call.

The split mirrors glTF / USD: a scene is a flat list of nodes (geometry,
viewpoints, lights), and the renderer is the orchestrator that walks the
scene and produces a frame. The `Scene` keeps the user-facing API in the
real-world layer; the GPU primitives (Pass, Shader, Texture) stay
underneath and don't leak into the call site.

## Lazy + sync, like Shader and Pass

`Scene::new()` is synchronous and takes no `Renderer`. The first time a
[SceneObject](https://fragmentcolor.org/api/scene) is added, the Scene
creates a default Pass to hold it. The first time the Scene is rendered, the
underlying GPU resources initialise on demand. This is the lazy-init pattern
the rest of FragmentColor follows.

## An open, composable pass graph

A `Scene` owns one ordered `Vec<Pass>`. Loaders, builders, and your own code
all append into the same list, and the renderer walks it in order. No pass is
privileged in render order: the default Pass that `scene.add(&model)` targets
is an ordinary member, slotted in at the point of your first `add`. After
[`Scene::load`](https://fragmentcolor.org/api/scene/scene/load) the whole
graph is in your hands. Read it with
[`list_passes`](https://fragmentcolor.org/api/scene/scene/list_passes) or
[`get_pass`](https://fragmentcolor.org/api/scene/scene/get_pass), restructure
it with [`add_pass`](https://fragmentcolor.org/api/scene/scene/add_pass),
[`remove_pass`](https://fragmentcolor.org/api/scene/scene/remove_pass), and
[`set_passes`](https://fragmentcolor.org/api/scene/scene/set_passes), and
configure each pass (`load_previous`, clear color, viewport, target) the same
way you would a hand-built one. That makes a loaded Scene compose onto any
other Pass in a frame, instead of clearing whatever it lands on.

## Default Camera + Light at render time

A Scene that only carries Models would normally render black: shaders need a
camera projection and at least one light for the lighting term to be
non-zero. To make the "hello world" path render something recognisable, the
Scene injects sensible defaults when the user hasn't supplied them:

- **Default Camera**: `Camera::perspective(60°, 1.0, 0.1, 100.0)` looking
  from `[0, 0, 5]` at the origin with `+Y` up.
- **Default Light**: a `Light::directional` aimed at
  `[0.0, -0.3, -1.0]` with full-white color, providing a forward-tilted
  fill so a front-facing quad reads as lit rather than silhouetted.

These only fire when no user Camera / Light has been added; as soon as you
add your own, your values win. A composition caller that drives every uniform
from the host can suppress them with
[`no_defaults`](https://fragmentcolor.org/api/scene/scene/no_defaults) (or the
per-kind `no_default_camera` / `no_default_light`), or swap the stock values
out with `set_default_camera` / `set_default_light`.

## Methods

| name                 | what it does                                                |
| -------------------- | ---------------------------------------------------------- |
| `new`                | construct an empty Scene                                   |
| `load`               | construct a Scene from a 3D file (glTF `.gltf` / `.glb` path or in-memory `.glb` bytes) |
| `add`                | add a `SceneObject` (Model / Camera / Light / custom) to the default pass |
| `add_to`             | add a `SceneObject` to a specific Pass, by index or name   |
| `add_pass`           | append a user-built [Pass](https://fragmentcolor.org/api/core/pass) to the graph |
| `remove_pass`        | remove a Pass from the graph by handle                     |
| `get_pass`           | read one Pass by index, in render order                    |
| `find_pass`          | find a Pass by name                                        |
| `list_passes`        | snapshot the whole pass graph in render order              |
| `set_passes`         | replace the whole pass graph with a new ordered list       |
| `no_defaults`        | suppress default-Camera and default-Light injection        |
| `no_default_camera`  | suppress default-Camera injection only                     |
| `no_default_light`   | suppress default-Light injection only                      |
| `set_default_camera` | inject your own Camera instead of the stock default        |
| `set_default_light`  | inject your own Light instead of the stock default         |
| `ambient`            | set the ambient color that lifts every Material's base lighting |

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Light, Material, Mesh, Model, Renderer, Scene, Vertex};

let renderer = Renderer::new();
let target = renderer.create_texture_target([256, 256]).await?;

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let model = Model::new(mesh, Material::pbr().base_color([0.8, 0.3, 0.2, 1.0]));

let camera = Camera::perspective(1.047, 1.0, 0.1, 100.0)
    .look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);

let scene = Scene::new();
scene
    .add(&model)?
    .add(&camera)?
    .add(&sun)?;

renderer.render(&scene, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
