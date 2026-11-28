# Model::material

The [Material](https://fragmentcolor.org/api/scene/material) the Model draws
with: the same handle `Model::new` took, so a setter on it changes every
Model that draws with that material.

In Rust it returns `&Material`, the enum with a variant per material type,
such as `Material::Pbr(PbrMaterial)`, `Material::Toon(ToonMaterial)` or
`Material::Custom(Shader)`, which a `match` or an `if let` reads. JavaScript,
Python, Swift and Kotlin return the material object itself, a `PbrMaterial`,
an `UnlitMaterial`, a `ToonMaterial` or a `Shader`, which each language narrows
as it narrows any value: `instanceof` in JavaScript, `isinstance` in Python,
`as?` in Swift and `is` or `as` in Kotlin. Swift types it as `any Material`, a
protocol each of those four conforms to. `Model::new` takes it back as it is.

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
