# Scene::list_passes

Return a snapshot of every [`Pass`](https://fragmentcolor.org/api/core/pass)
in the scene, in render order. Each entry is an Arc-shared clone, so
configuring one (`load_previous`, `set_clear_color`, `set_viewport`,
`set_target`) drives the pass the Scene renders. The returned `Vec` is the
snapshot at call time; appending passes afterward doesn't grow it.

This is the composition hook. After
[`Scene::load`](https://fragmentcolor.org/api/scene/scene/load) the whole
pass graph is in your hands: walk it and call `load_previous` so the scene
composes onto a previous draw instead of clearing it, then hand the Scene to
the renderer alongside whatever else the frame needs.

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

// Compose, don't clear: keep whatever the previous pass drew.
for pass in scene.list_passes() {
    pass.load_previous();
}
# Ok(())
# }
```
