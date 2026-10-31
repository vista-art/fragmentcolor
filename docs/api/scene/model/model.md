# Model

A `Model` pairs a [Mesh](https://fragmentcolor.org/api/geometry/mesh) with a
[Material](https://fragmentcolor.org/api/scene/material) and adds a per-Model
4×4 transform. It's the unit you actually add to a `Pass` when rendering 3D
content; the Material handles shading, the Mesh handles geometry, the Model
handles "where in the world".

Each `Model::new` takes ownership of the Material it's given. Cloning the
Material before passing it in is cheap: `Material::clone` shares the handle
and copies nothing. The per-Model transform is *not* a Material uniform. The
renderer reads it at draw time and feeds it to the vertex stage as four
`vec4<f32>` instance attributes, `model_0` to `model_3`, the columns of a
`mat4x4<f32>`. The PBR shader reads them at locations 3 to 6.

Models that share a Material and a Mesh draw together. On every render, the
renderer packs the transforms of the visible `Opaque` and `Mask` Models with
the same Shader and Mesh into one instance buffer and draws them in one
instanced call. `Blend` Models draw one at a time, back to front, since
their order matters.

For the fan-out case (one Material, many positioned items), the pattern is:

```text
template = Material::pbr().base_color(...)
for each item:
  model = Model::new(item.mesh, template.clone())
  model.set_transform(item.matrix)
  pass.add(&model)
```

The pipeline is cached once per shader, the bind groups are set once per
pass, and the pass issues one draw per Mesh. Give every Model the same
`mesh.clone()` to draw them all in one call: `Mesh::clone` shares the
handle, so the Models share one Mesh. A transform changed after
`pass.add(&model)` shows up on the next render, without adding the Model
again.

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
| `visible`      | read whether the Model draws                       |
| `set_visible`  | show or hide the Model without removing it         |

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Vertex};

let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
]);

let mat = Material::pbr().base_color([0.3, 0.6, 1.0, 1.0]);
let model = Model::new(mesh, mat);
model.translate([2.0, 0.0, 0.0]);
model.scale([1.5, 1.5, 1.5]);

# Ok(())
# }
```
