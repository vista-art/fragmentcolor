# Pass::add_model

Add a [Model](https://fragmentcolor.org/api/scene/model) to this render pass.
This is the high-level shortcut that handles the three operations a manual
attachment would need:

1. Push the Model's Material shader onto the pass.
2. Attach the Model's Mesh to that shader.
3. The Model already keeps its `mesh.model` uniform synchronised with its
   transform, so the freshly-queued draw uses the current position.

Each Model carries its own Shader (cloned from the Material at construction
time) so per-Model transforms don't collide when many Models share the same
template Material. For RemixBrush-style "279 blobs with one PBR look,
different positions" you create one Material, clone it into each Model, set
the Model's transform, and `add_model` each one. That's 279 draw calls
today; the API doesn't change when batched instancing lands.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Pass, Renderer, Vertex};

let renderer = Renderer::new();
let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.5, 1.0]),
);
mesh.add_vertex(
    Vertex::new([-0.5, -0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.0, 0.0]),
);
mesh.add_vertex(
    Vertex::new([0.5, -0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [1.0, 0.0]),
);

let template = Material::pbr(&renderer).await?.base_color([0.85, 0.4, 0.2, 1.0]);
let pass = Pass::new("scene");

let m1 = Model::new(mesh.clone(), template.clone());
m1.translate([-1.0, 0.0, 0.0]);
pass.add_model(&m1)?;

let m2 = Model::new(mesh, template);
m2.translate([1.0, 0.0, 0.0]);
pass.add_model(&m2)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
