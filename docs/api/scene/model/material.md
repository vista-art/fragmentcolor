# Model::material

The [Material](https://fragmentcolor.org/api/scene/material) the Model draws
with: the same handle `Model::new` took, so a setter on it changes every
Model that draws with that material.

In Rust it returns `&Material`, the enum with a variant per material type,
such as `Material::Pbr(PbrMaterial)`, `Material::Unlit(UnlitMaterial)` or
`Material::Custom(Shader)`. JavaScript and Python return the material object
itself, a `PbrMaterial`, an `UnlitMaterial` or a `Shader`. Swift and Kotlin return the `Material` enum, which holds the
object.

To give a Model a look of its own, build a fresh material, such as a new
`PbrMaterial::new()`, rather than cloning one.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mesh, Model, PbrMaterial, Vertex};

let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
]);

let model = Model::new(mesh, PbrMaterial::new());
let material = model.material();
# let _ = material;
# Ok(())
# }
```
