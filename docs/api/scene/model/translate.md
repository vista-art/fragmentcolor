# Model::translate

Move the Model by `offset` in world coordinates. Pre-multiplies the current
transform by a translation matrix, so the result is independent of the
Model's current rotation. `translate([1, 0, 0])` always moves one unit
along the world X axis.

An item that is NaN or an infinity returns `ModelError::NotFinite`, which
names the call and the item, and the Model stays where it was. The other
languages also refuse an item past the 32-bit float range, such as `1e39`,
with a message like `Model.translate: item 1 must be a finite number that
fits a 32-bit float, got 1e39`. JavaScript throws an `Error` named
`InvalidInputError`, Python raises `InvalidInputError`, Swift throws
`FragmentColorError` and Kotlin throws `FragmentColorException`.

A move of finite numbers that would carry a number of the transform past
the 32-bit float range, such as two moves of `3e38` along one axis, returns
`ModelError::Overflow`: `Model::translate would carry the transform past the f32
range`. The Model keeps its transform, and JavaScript and Python raise it as
`InvalidInputError`, too.

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
model.translate([5.0, 0.0, -2.0])?;

# let m = model.transform();
# // The translation lives in the fourth column when column-major.
# assert_eq!(m[3], [5.0, 0.0, -2.0, 1.0]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
