# Model::scale

Scale the Model by per-axis `factor` (in local space). Post-multiplies the
current transform by a scale matrix, so the Model grows or shrinks around
its own origin without sliding through space.

Use uniform scales (`[s, s, s]`) when possible. Non-uniform scale breaks
the cheap normal-transform path the default PBR shader uses, so a stretched
Model will shade slightly off. For a correct non-uniform-scale normal, use
`Material::custom` with a shader that ships the explicit cofactor matrix.

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
model.scale([2.0, 2.0, 2.0]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
