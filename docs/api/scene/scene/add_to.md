# Scene::add_to

Add a [`SceneObject`](https://fragmentcolor.org/api/scene) (Model / Camera /
Light / custom) to a specific Pass in the graph, addressed by index or by
name. It's the targeted form of
[`add`](https://fragmentcolor.org/api/scene/scene/add): where `add` routes
into the scene's default pass, `add_to` lets you pick the pass yourself, which
matters once a scene holds more than one.

Pass an index (matching [`get_pass`](https://fragmentcolor.org/api/scene/scene/get_pass))
or a name (matching [`find_pass`](https://fragmentcolor.org/api/scene/scene/find_pass)).
An index out of range, or a name that matches nothing, is an error. The
Scene-level bookkeeping is identical to `add`: the object surfaces in
`models()` / `cameras()` / `lights()`, and a Camera or Light there still
suppresses the matching default-injection.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Pass, Scene, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let model = Model::new(mesh, Material::pbr());

let scene = Scene::new();
scene.add_pass(&Pass::new("geometry"));

// Target the pass by name (or pass its index: scene.add_to(0, &model)).
scene.add_to("geometry", &model)?;
# assert_eq!(scene.models().len(), 1);
# Ok(())
# }
```
