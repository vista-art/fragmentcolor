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
creates a default Pass to absorb it. The first time the Scene is rendered,
the underlying GPU resources initialise on demand. This is the lazy-init
pattern the rest of FragmentColor follows.

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
add your own, your values win.

## Methods

| name        | what it does                                                |
| ----------- | ----------------------------------------------------------- |
| `new`       | construct an empty Scene                                    |
| `load`      | construct a Scene from a 3D file (glTF `.gltf` / `.glb` path or in-memory `.glb` bytes) |
| `add`       | absorb a `SceneObject` (Model / Camera / Light / custom)    |
| `add_pass`  | absorb a user-built [Pass](https://fragmentcolor.org/api/core/pass) for explicit pipeline ordering (shadows, post-fx, …) |
| `ambient`   | set the ambient color that lifts every Material's base lighting |

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Light, Material, Mesh, Model, Renderer, Scene, Vertex};

let renderer = Renderer::new();
let target = renderer.create_texture_target([256u32, 256u32]).await?;

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let model = Model::new(mesh, Material::pbr()?.base_color([0.8, 0.3, 0.2, 1.0]));

let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
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
