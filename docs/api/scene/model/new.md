# Model::new

Construct a `Model` from a `Mesh` and a `Material`. Both arguments are taken
by value; the Model owns them. The transform starts at the 4×4 identity
matrix. The renderer reads it at draw time and passes it to the vertex stage
as the `model_0` to `model_3` instance attributes, the four columns of a
`mat4x4<f32>`.

If you want several Models that share a look, clone the Material before each
`Model::new`. `Material::clone` shares the handle and copies nothing, so the
Models share one bind-group setup, and Models whose meshes lay out their
vertices alike share one pipeline. The transform belongs to
the Model, not to the Material's Shader or the Mesh, so Models can share
both. Models with the same Material and Mesh draw in one instanced call.

`Material::pbr` reads six attributes from every vertex: a 3D position
(`vec3`), `normal` (`vec3`), `uv0` (`vec2`), `color0` (`vec4`), `uv1`
(`vec2`) and `tangent` (`vec4`). Each input finds its attribute by name,
then by the attribute's own location, as
[Mesh](https://fragmentcolor.org/api/geometry/mesh) describes, so a vertex
that sets them under these names can set them in any order. The shader has
no default for any of them, and `Scene.add` and `Pass.add` refuse a Model
whose mesh lacks one. The error names the input, or the attribute at the
input's location when that attribute has the wrong type: a vertex that sets
only `uv0` fails on `uv0`, which sits where `normal` is read. `Vertex::pbr`
sets all five after the position to neutral values, and `.set(...)`
replaces the ones your mesh has data for. The `model_0` to `model_3`
columns come from the Model, not the mesh. Custom shaders via
`Material::custom(...)` can use any layout.

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

# let _model = model;
# Ok(())
# }
```
