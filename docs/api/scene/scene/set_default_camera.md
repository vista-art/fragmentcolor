# Scene::set_default_camera

Supply your own default [`Camera`](https://fragmentcolor.org/api/scene/camera)
for the Scene to inject at first render in place of FC's stock perspective
camera. This fires only when no Camera arrived through
[`Scene::add`](https://fragmentcolor.org/api/scene/scene/add); an explicit
`scene.add(&camera)` still wins.

Naming a default re-arms injection. If
[`no_default_camera`](https://fragmentcolor.org/api/scene/scene/no_default_camera)
ran earlier, calling `set_default_camera` turns camera injection back on with
the camera you pass.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Material, Mesh, Model, Scene, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let scene = Scene::new();
scene.add(&Model::new(mesh, Material::pbr()))?;

let camera = Camera::perspective(1.047, 16.0 / 9.0, 0.1, 100.0)
    .look_at([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
scene.set_default_camera(&camera);
# Ok(())
# }
```
