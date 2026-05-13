# Model::rotate

Rotate the Model around `axis` (in local space) by `radians`. Post-multiplies
the current transform by an axis-angle rotation, so a rotated-then-translated
Model spins in place around its own origin rather than orbiting the world
origin. The axis is normalised internally; a zero-length axis is rejected
with a debug log and no change.

For pure world-space rotations (e.g. tumbling around a fixed pivot), compose
the matrix yourself and call [set_transform](https://fragmentcolor.org/api/scene/model#modelset_transform).

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
model.rotate([0.0, 1.0, 0.0], std::f32::consts::FRAC_PI_2);
# Ok(())
# }
```
