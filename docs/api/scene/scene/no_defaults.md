# Scene::no_defaults

Turn off both default-Camera and default-Light injection. By default a Scene
carrying Models but no Camera or Light injects a stock perspective camera and
a white directional light at first render, so the hello-world path shows
something recognisable. A composition caller that drives `view_proj` and the
light slots from the host doesn't want those stock values layered on top.
`no_defaults` leaves the pass graph exactly as built.

For finer control, reach for the per-kind switches
[`no_default_camera`](https://fragmentcolor.org/api/scene/scene/no_default_camera)
and [`no_default_light`](https://fragmentcolor.org/api/scene/scene/no_default_light).
To keep injection on but swap the stock values for your own, use
[`set_default_camera`](https://fragmentcolor.org/api/scene/scene/set_default_camera)
and [`set_default_light`](https://fragmentcolor.org/api/scene/scene/set_default_light).

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Scene, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let scene = Scene::new();
scene.add(&Model::new(mesh, Material::pbr()))?;

// The host overrides every uniform, so suppress FC's stock camera + light.
scene.no_defaults();
for p in scene.list_passes() {
    p.load_previous();
}
# Ok(())
# }
```
