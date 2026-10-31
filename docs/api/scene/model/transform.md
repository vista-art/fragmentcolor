# Model::transform

Read the Model's current 4×4 transform as its four columns of four numbers,
the `[[f32; 4]; 4]` Rust returns, and four lists of four in every other language
(four `Float32Array`s in JavaScript).
Column-major order matches WGSL's `mat4x4<f32>` layout, so `transform()[3]` holds
the translation. The transform starts at the identity matrix and is modified by
`set_transform`, `translate`, `rotate`, and `scale`.

For the setter, see [Model::set_transform](https://fragmentcolor.org/api/scene/model#set_transform).
Rust doesn't allow getter/setter overloads on the same name, so the read
side is `transform` and the write side is `set_transform`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Renderer, Vertex};

let renderer = Renderer::new();
let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
]);

let model = Model::new(mesh, Material::pbr());
let identity = model.transform();
# assert_eq!(identity[0], [1.0, 0.0, 0.0, 0.0]);
# assert_eq!(identity[3], [0.0, 0.0, 0.0, 1.0]);
# let _ = identity;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
