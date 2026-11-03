# Model::set_transform

Replace the Model's 4×4 transform wholesale, in column-major order. Useful
when you already have a matrix from a math library or a glTF node and want
to apply it directly without composing through `translate` / `rotate` /
`scale`.

The renderer reads the new matrix on the next render, so a Model already
added to a Pass or Scene moves without being added again. To read the
current transform, see
[Model::transform](https://fragmentcolor.org/api/scene/model#transform).

A number that is NaN or an infinity returns `ModelError::NotFinite`, which
names the call, the column and the item: `Model::set_transform: column 3:
item 0 must be finite, got inf`. The Model keeps the transform it had. The
other languages also refuse a number past the 32-bit float range, such as
`1e39`, with a message like `Model.setTransform: column 3: item 0 must be a
finite number that fits a 32-bit float, got 1e39`, or `item 12` for the 16
numbers of a flat matrix. JavaScript throws an `Error` named
`InvalidInputError`, Python raises `InvalidInputError`, Swift throws
`FragmentColorError` and Kotlin throws `FragmentColorException`.

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
model.set_transform([
    [2.0, 0.0, 0.0, 0.0],
    [0.0, 2.0, 0.0, 0.0],
    [0.0, 0.0, 2.0, 0.0],
    [3.0, 0.0, 0.0, 1.0],
])?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
