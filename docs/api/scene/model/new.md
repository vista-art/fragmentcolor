# Model::new

Construct a `Model` from a `Mesh` and a `Material`. Both arguments are taken
by value — the Model owns them. The transform starts at the 4×4 identity
matrix and the Material's `mesh.model` uniform is initialised to match.

If you want several Models that share a look, clone the Material before each
`Model::new` — `Material::clone` is a deep clone that gives each Model its
own independent shader copy, so per-Model transforms don't collide.

`Material::pbr()` requires the Mesh's first vertex to declare position
(`vec3`), normal (`vec3`), and uv0 (`vec2`) in that exact insertion order,
so the locations align with the PBR shader's vertex inputs. Custom shaders
via `Material::custom(...)` can use any layout.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Vertex};

let mut mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.0, 0.0])
        .set(Vertex::NORMAL, [0.0, 1.0, 0.0])
        .set(Vertex::UV0, [0.0, 0.0]),
);

let model = Model::new(mesh, Material::pbr());

# let _ = model;
# Ok(())
# }
```
