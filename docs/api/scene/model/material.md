# Model::material

Borrow the Material handle the Model owns. The Material wraps a Shader;
reach through `model.material().shader()` if you need direct uniform
manipulation (camera state, custom uniforms not exposed by Material's
setters).

Note: `Material::clone` is a shallow Arc-share, so the handle stored on this
Model points at the same underlying shader state as whatever Material was
passed to `Model::new`. Mutations made through `model.material()` are
visible to every other Model that received the same source handle (and vice
versa). To detach a Model's appearance, construct a fresh `Material::pbr()`
or `Material::custom()` rather than cloning.

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

let model = Model::new(mesh, Material::pbr());
model.material().shader().set("camera.position", [0.0, 0.0, 5.0])?;
# Ok(())
# }
```
