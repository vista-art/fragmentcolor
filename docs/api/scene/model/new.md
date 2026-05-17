# Model::new

Construct a `Model` from a `Mesh` and a `Material`. Both arguments are taken
by value — the Model owns them. The transform starts at the 4×4 identity
matrix; `Model::sync_transform` writes it as a single instance into the
Mesh's per-instance attribute stream (four `vec4<f32>` columns at locations
3..6).

If you want several Models that share a look, clone the Material before each
`Model::new`. `Material::clone` is a cheap Arc-clone (handle share, not a
deep duplicate), so the Models share one pipeline + one bind-group setup.
Per-Model transforms don't collide because each Model owns its own Mesh, and
the transform lives on the Mesh's instance buffer, not the Material's Shader.

`Material::pbr` requires the Mesh's first vertex to declare position
(`vec3`), normal (`vec3`), and uv0 (`vec2`) in that exact insertion order,
so the locations align with the PBR shader's vertex inputs. Custom shaders
via `Material::custom(...)` can use any layout.

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

# let _model = model;
# Ok(())
# }
```
