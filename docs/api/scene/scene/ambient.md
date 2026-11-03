# Scene::ambient

Set the scene-wide ambient color. The PBR shader adds
`albedo * ambient` to every fragment regardless of which lights are
attached, so unlit faces don't read pitch-black and lit faces get a
subtle warm/cool fill on top of direct lighting.

Defaults: every Material seeded by `Material::pbr()` starts with
`ambient = [0.03, 0.03, 0.03]` (the dim gray the PBR shader had
hardcoded before this knob existed). Calling `Scene::ambient` overrides
that for every shader the scene visits today *and* for any Models added
afterwards: the value is stashed on the Scene and re-stamped onto
shaders that join later via `Scene::add`.

A shader whose `lights.ambient` can't hold the color, such as an `f16`
ambient and a channel past ±65504, keeps the color it had, and
`Renderer::render` returns the shader's `ShaderError::TypeMismatch` until a
color lands there, from a later `ambient` call or from `Shader::set`. A
shader without `lights.ambient` takes nothing.

A channel that is NaN or an infinity returns `LightError::NotFinite`,
which names the call and the item, `Scene::ambient: item 1 must be finite,
got NaN`, and the scene and its shaders keep the color they had. The
other languages also refuse a channel past the 32-bit float range, such as
`1e39`. JavaScript throws an `Error` named `InvalidInputError`, Python
raises `InvalidInputError`, Swift throws `FragmentColorError` and Kotlin
throws `FragmentColorException`.

Returns `Ok` with a handle to the same Scene (Arc-shared) for chaining.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Light, Material, Mesh, Model, Renderer, Scene, Vertex};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
]);

let scene = Scene::new();
// A warm dusk ambient. It applies to every Material added below.
scene.ambient([0.06, 0.04, 0.03])?;
scene.add(&Model::new(mesh, Material::pbr()))?;
scene.add(&Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])?)?;

renderer.render(&scene, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
