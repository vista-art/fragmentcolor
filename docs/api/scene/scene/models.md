# Scene::models

Return a snapshot of every [`Model`](https://fragmentcolor.org/api/scene/model)
added to this Scene via [`Scene::add`](https://fragmentcolor.org/api/scene/scene/add),
including Models the loader instantiated from glTF `mesh` nodes.

Each entry is an Arc-shared clone of the original handle. Mutating one
of them (`set_visible`, `translate`, `set_transform`, …) propagates live
to every shader the Model was wired into, no re-attach needed. The
returned `Vec` is the *snapshot at call time*. Adding more Models after
the call doesn't grow this `Vec`, but the handles you already have stay
live.

Order: insertion order. Loader-produced Models appear in glTF node-walk
order; user-added Models appear after.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Scene, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let model = Model::new(mesh, Material::pbr());

let scene = Scene::new();
scene.add(&model)?;

// LOD switch: hide every model the user just loaded, based on a
// camera-distance heuristic the caller computes elsewhere.
for m in scene.models() {
    m.set_visible(false);
}
# Ok(())
# }
```
