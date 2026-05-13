# Model::set_transform

Replace the Model's 4×4 transform wholesale, in column-major order. Useful
when you already have a matrix from a math library or a glTF node and want
to apply it directly without composing through `translate` / `rotate` /
`scale`.

Writes the four columns into the Mesh's per-instance attribute stream
immediately (locations 3..6). To read the current transform, see
[Model::transform](https://fragmentcolor.org/api/scene/model#modeltransform).

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Renderer, Vertex};

let renderer = Renderer::new();
let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.0, 0.0])
        .set(Vertex::NORMAL, [0.0, 1.0, 0.0])
        .set(Vertex::UV0, [0.0, 0.0]),
);

let model = Model::new(mesh, Material::pbr()?);
model.set_transform([
    [2.0, 0.0, 0.0, 0.0],
    [0.0, 2.0, 0.0, 0.0],
    [0.0, 0.0, 2.0, 0.0],
    [3.0, 0.0, 0.0, 1.0],
]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
