# Model::new

Construct a `Model` from a `Mesh` and a `Material`. Both arguments are taken
by value; the Model owns them. The transform starts at the 4×4 identity
matrix. The renderer reads it at draw time and passes it to the vertex stage
as the `model_0` to `model_3` instance attributes, the four columns of a
`mat4x4<f32>`.

If you want several Models that share a look, clone the Material before each
`Model::new`. `Material::clone` shares the handle and copies nothing, so the
Models share one pipeline and one bind-group setup. The transform belongs to
the Model, not to the Material's Shader or the Mesh, so Models can share
both. Models with the same Material and Mesh draw in one instanced call.

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

let model = Model::new(mesh, Material::pbr());

# let _model = model;
# Ok(())
# }
```
