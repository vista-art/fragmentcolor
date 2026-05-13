# Model::transform

Read the Model's current 4×4 transform in column-major order, matching
WGSL's `mat4x4<f32>` layout and glam's `to_cols_array_2d`. The transform
starts at the identity matrix and is modified by `set_transform`,
`translate`, `rotate`, and `scale`.

For the setter, see [Model::set_transform](https://fragmentcolor.org/api/scene/model#modelset_transform).
Rust doesn't allow getter/setter overloads on the same name, so the read
side is `transform` and the write side is `set_transform`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.0, 0.0])
        .set(Vertex::NORMAL, [0.0, 1.0, 0.0])
        .set(Vertex::UV0, [0.0, 0.0]),
);

let model = Model::new(mesh, Material::pbr()?);
let identity = model.transform();
# assert_eq!(identity[0], [1.0, 0.0, 0.0, 0.0]);
# assert_eq!(identity[3], [0.0, 0.0, 0.0, 1.0]);
# let _ = identity;
# Ok(())
# }
```
