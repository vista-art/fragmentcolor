# Model::scale

Scale the Model by per-axis `factor` (in local space). Post-multiplies the
current transform by a scale matrix, so the Model grows or shrinks around
its own origin without sliding through space.

Unequal factors are fine: the built-in materials light a stretched Model
the way its surfaces face. A negative factor mirrors the Model, which turns
its faces' winding around on screen. A single-sided material then culls the
outer faces, so give a mirrored Model a double-sided material, which lights
them as the front.

A factor that is NaN or an infinity returns `ModelError::NotFinite`, which
names the call and the item, and the Model keeps its size. The other
languages also refuse a factor past the 32-bit float range, such as
`1e39`. JavaScript throws an `Error` named `InvalidInputError`, Python
raises `InvalidInputError`, Swift throws `FragmentColorError` and Kotlin
throws `FragmentColorException`. A factor of 0 is a valid scale, which
flattens the Model along that axis.

A scale that would carry a number of the transform past the 32-bit float
range, such as `[1e20, 1e20, 1e20]` twice, returns `ModelError::Overflow`,
`Model::scale would carry the transform past the f32 range`, and the Model
keeps its size. JavaScript and Python raise it as `InvalidInputError`, too.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mesh, Model, PbrMaterial, Renderer, Vertex};

let renderer = Renderer::new();
let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
]);

let model = Model::new(mesh, PbrMaterial::new());
model.scale([2.0, 2.0, 2.0])?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
