# Scene::set_default_light

Supply your own default [`Light`](https://fragmentcolor.org/api/scene/light)
for the Scene to inject at first render in place of FC's stock white
directional light. This fires only when no Light arrived through
[`Scene::add`](https://fragmentcolor.org/api/scene/scene/add); an explicit
`scene.add(&light)` still wins.

Naming a default re-arms injection. If
[`no_default_light`](https://fragmentcolor.org/api/scene/scene/no_default_light)
ran earlier, calling `set_default_light` turns light injection back on with
the light you pass.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Light, Material, Mesh, Model, Scene, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let scene = Scene::new();
scene.add(&Model::new(mesh, Material::pbr()))?;

let key = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
scene.set_default_light(&key);
# Ok(())
# }
```
