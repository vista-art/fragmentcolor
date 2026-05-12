# Model

A `Model` pairs a [Mesh](https://fragmentcolor.org/api/geometry/mesh) with a
[Material](https://fragmentcolor.org/api/scene/material) and adds a per-Model
4×4 transform. It's the unit you actually add to a `Pass` when rendering 3D
content; the Material handles shading, the Mesh handles geometry, the Model
handles "where in the world".

Each `Model::new` takes ownership of the Material it's given and stores it
as-is. The Material's `shader` carries the Model's `mesh.model` uniform —
modifying the Model's transform writes to that uniform on its own shader.
If you want N Models with the same look and different transforms, clone the
template Material into each: `Material::clone` is a deep clone that spawns
an independent shader copy, so per-Model transforms don't collide.

For RemixBrush-style fan-out (one Material template, many positioned
instances), the pattern is:

```text
template = Material::pbr().base_color(...)
for each blob:
  model = Model::new(blob.mesh, template.clone())
  model.set_transform(blob.matrix)
  pass.add_model(&model)
```

That's N draw calls today (one per Model). Phase 2 batches Models sharing
the same Material into one instanced draw — the API doesn't change.

## Methods

| name           | what it does                                       |
| -------------- | -------------------------------------------------- |
| `new`          | construct a Model from a Mesh + Material          |
| `mesh`         | borrow the Mesh                                    |
| `material`     | borrow the Material                                |
| `transform`    | read the current 4×4 (column-major)                |
| `set_transform`| replace the 4×4                                    |
| `translate`    | pre-multiply by a world-space translation         |
| `rotate`       | post-multiply by a local-space axis-angle rotation |
| `scale`        | post-multiply by a local-space per-axis scale     |

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Vertex};

let mut mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.5, 1.0]),
);

let mat = Material::pbr().base_color([0.3, 0.6, 1.0, 1.0]);
let model = Model::new(mesh, mat);
model.translate([2.0, 0.0, 0.0]);
model.scale([1.5, 1.5, 1.5]);

# Ok(())
# }
```
