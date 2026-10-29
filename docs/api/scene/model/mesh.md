# Model::mesh

Borrow the Mesh the Model owns. Useful for adding more vertices or instances
to the geometry after construction without losing access through the Model
handle.

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

let model = Model::new(mesh, Material::pbr());
// A second triangle joins the first through the Model.
model.mesh().add_vertices([
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
    Vertex::pbr([0.5, 0.5, 0.0]).set(Vertex::UV0, [1.0, 1.0]),
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
]);
# Ok(())
# }
```
