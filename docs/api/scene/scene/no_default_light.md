# Scene::no_default_light

Turn off default-Light injection while leaving the default Camera in place.
By default a Scene with Models but no Light injects a white directional light
at first render so the geometry reads as lit. Call this when the host owns the
light rig and the stock light would only compete for the same
`lights.lights[..]` slots.

To turn off both kinds at once, use
[`no_defaults`](https://fragmentcolor.org/api/scene/scene/no_defaults). To
keep injection on but supply your own light, use
[`set_default_light`](https://fragmentcolor.org/api/scene/scene/set_default_light).

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

scene.no_default_light();
# Ok(())
# }
```
