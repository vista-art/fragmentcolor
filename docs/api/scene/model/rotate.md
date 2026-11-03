# Model::rotate

Rotate the Model around `axis` (in local space) by `radians`. Post-multiplies
the current transform by an axis-angle rotation, so a rotated-then-translated
Model spins in place around its own origin rather than orbiting the world
origin. The axis is normalized internally, so an axis of any length but 0
turns the Model the same way.

An axis item or an angle that is NaN or an infinity returns
`ModelError::NotFinite`, which names the call and the item, such as
`Model::rotate: radians must be finite, got NaN`. The other languages also
refuse a number past the 32-bit float range, such as `1e39`, with a
message like `Model.rotate: axis: item 1 must be a finite number that fits
a 32-bit float, got 1e39`. An axis of length 0, such as `[0, 0, 0]`, has
no direction to turn around and returns `ModelError::ZeroAxis` in every
language. Either way the Model keeps its transform. JavaScript throws an
`Error` named `InvalidInputError`, Python raises `InvalidInputError`,
Swift throws `FragmentColorError` and Kotlin throws
`FragmentColorException`.

For pure world-space rotations (e.g. tumbling around a fixed pivot), compose
the matrix yourself and call [set_transform](https://fragmentcolor.org/api/scene/model#set_transform).

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Renderer, Vertex};

let renderer = Renderer::new();
let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
]);

let model = Model::new(mesh, Material::pbr());
model.rotate([0.0, 1.0, 0.0], 1.571)?; // 90° around Y
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
