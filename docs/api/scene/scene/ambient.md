# Scene::ambient

Set the scene-wide ambient color. The PBR shader adds
`albedo * ambient` to every fragment regardless of which lights are
attached, so unlit faces don't read pitch-black and lit faces get a
subtle warm/cool fill on top of direct lighting.

Defaults: every Material seeded by `Material::pbr()` starts with
`ambient = [0.03, 0.03, 0.03]` (the dim grey the PBR shader had
hardcoded before this knob existed). Calling `Scene::ambient` overrides
that for every shader the scene visits today *and* for any Models added
afterwards: the value is stashed on the Scene and re-stamped onto
shaders that join later via `Scene::add`.

Returns a handle to the same Scene (Arc-shared) for chaining.

## Example

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Light, Material, Mesh, Model, Renderer, Scene, Vertex};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64u32, 64u32]).await?;

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);

let scene = Scene::new();
// Warm dusk ambient — applies to every Material added below.
scene.ambient([0.06, 0.04, 0.03]);
scene.add(&Model::new(mesh, Material::pbr()?))?;
scene.add(&Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]))?;

renderer.render(&scene, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
