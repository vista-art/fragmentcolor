# Model::material

Borrow the Material the Model owns. The Material wraps a Shader; reach
through `model.material().shader()` if you need direct uniform manipulation
(camera state, custom uniforms not exposed by Material's setters).

Note: this returns the Model's *own* Material (independent of whatever
template was passed to `Model::new`), so tweaking it affects this Model
alone.

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
model.material().shader().set("camera.position", [0.0_f32, 0.0, 5.0])?;
# Ok(())
# }
```
