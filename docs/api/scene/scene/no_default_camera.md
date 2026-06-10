# Scene::no_default_camera

Turn off default-Camera injection while leaving the default Light in place.
By default a Scene with Models but no Camera injects a stock perspective
camera at first render. Call this when the host supplies the view projection
and the stock camera would only fight for the same uniform.

To turn off both kinds at once, use
[`no_defaults`](https://fragmentcolor.org/api/scene/scene/no_defaults). To
keep injection on but supply your own camera, use
[`set_default_camera`](https://fragmentcolor.org/api/scene/scene/set_default_camera).

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

scene.no_default_camera();
# Ok(())
# }
```
