# Scene::add_pass

Append a user-built [Pass](https://fragmentcolor.org/api/core/pass) to the
scene's pre-pass list. The Scene renders these passes in insertion order
*before* its internal default Pass — that's the hook for shadow maps,
depth pre-passes, screen-space backdrops, anything that has to run *before*
the scene's main geometry draw.

The default Pass that holds the Scene's [SceneObjects](https://fragmentcolor.org/api/scene)
is *not* affected by `add_pass`; it's an internal pipeline owned by the
Scene. Use `add_pass` only when you want a dedicated Pass with its own
shaders / clear logic — for ordinary geometry, `scene.add(&model)?` is what
you want.

For *post*-effects (bloom, tonemap, …), keep the post-pass outside the
Scene and combine them at render time:

```text
renderer.render(&scene, &offscreen_target)?;
renderer.render(&postfx_pass, &final_target)?;
```

The Pass is cloned (shallow Arc-share) when stored, so further changes you
make to the original handle reach the Scene's copy as well.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Pass, Renderer, Scene, Vertex};

let renderer = Renderer::new();

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.5, 1.0]),
);
let model = Model::new(mesh, Material::pbr()?);

// A backdrop pass that clears to a soft blue before the scene's main draw.
let backdrop = Pass::new("backdrop");
backdrop.set_clear_color([0.05, 0.08, 0.12, 1.0]);

let scene = Scene::new();
scene.add_pass(&backdrop);
scene.add(&model)?;

# let _ = renderer;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
