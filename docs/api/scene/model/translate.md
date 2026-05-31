# Model::translate

Move the Model by `offset` in world coordinates. Pre-multiplies the current
transform by a translation matrix, so the result is independent of the
Model's current rotation. `translate([1, 0, 0])` always moves one unit
along the world X axis.

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
model.translate([5.0, 0.0, -2.0]);

# let m = model.transform();
# // The translation lives in the fourth column when column-major.
# assert_eq!(m[3], [5.0, 0.0, -2.0, 1.0]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
