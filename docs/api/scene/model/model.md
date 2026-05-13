# Model

A `Model` pairs a [Mesh](https://fragmentcolor.org/api/geometry/mesh) with a
[Material](https://fragmentcolor.org/api/scene/material) and adds a per-Model
4×4 transform. It's the unit you actually add to a `Pass` when rendering 3D
content; the Material handles shading, the Mesh handles geometry, the Model
handles "where in the world".

Each `Model::new` takes ownership of the Material it's given. Cloning the
Material before passing it in is cheap — `Material::clone` is an Arc-clone
(handle share), not a deep duplicate. The per-Model transform is *not* a
Material uniform: it's written as four `vec4<f32>` columns into the Mesh's
per-instance attribute stream at locations 3..6. Many Models can share one
Material's Shader without colliding because each Model writes its transform
to **its own** Mesh's instance buffer.

For RemixBrush-style fan-out (one Material, many positioned instances on
unique geometries), the pattern is:

```text
template = Material::pbr().base_color(...)
for each blob:
  model = Model::new(blob.mesh, template.clone())
  model.set_transform(blob.matrix)
  pass.add(&model)
```

Pipeline cached once by shader hash; one bind-group setup per pass; N draws
(one per unique Mesh). For *batched* instancing — one shared Mesh, many
transforms in one draw — drop down to `Mesh::add_instance(...)` directly
with a `Material::custom(shader_that_reads_instance_attrs)`. The Model API
is for one logical thing per draw, not for managing your own per-instance
buffers.

**Caveat:** the Mesh's instance buffer is Arc-shared via `Mesh::clone`. Two
Models that share a Mesh handle (`mesh.clone()`) collide on the same instance
buffer — the most recent transform-mutating call wins. Give each Model its
own Mesh.

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
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Renderer, Vertex};

let renderer = Renderer::new();
let mut mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.5, 1.0]),
);

let mat = Material::pbr()?.base_color([0.3, 0.6, 1.0, 1.0]);
let model = Model::new(mesh, mat);
model.translate([2.0, 0.0, 0.0]);
model.scale([1.5, 1.5, 1.5]);

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
