# Scene::add_pass

Append a user-built [Pass](https://fragmentcolor.org/api/core/pass) to the
scene's pre-pass list. The Scene renders these passes in insertion order
*before* its internal default Pass. That's the hook for shadow maps,
depth pre-passes, screen-space backdrops, anything that has to run *before*
the scene's main geometry draw.

The default Pass that holds the Scene's [SceneObjects](https://fragmentcolor.org/api/scene)
is *not* affected by `add_pass`; it's an internal pipeline owned by the
Scene. Use `add_pass` only when you want a dedicated Pass with its own
shaders / clear logic. For ordinary geometry, `scene.add(&model)?` is what
you want.

For *post*-effects (bloom, tonemap, …), keep the post-pass outside the
Scene and combine them at render time:

```text
renderer.render(&scene, &offscreen_target)?;
renderer.render(&postfx_pass, &final_target)?;
```

The Pass is cloned (shallow Arc-share) when stored, so further changes you
make to the original handle reach the Scene's copy as well.

### Each new Pass clears by default

Every `Pass::new(name)` starts with a clear-to-transparent input. The
first render of the frame for that Pass wipes its colour attachment to
`[0, 0, 0, 0]`. When you chain passes that read each other's output
(e.g. `[backdrop, shadow_overlay]`), the second Pass will clear the
first Pass's output unless you opt out:

- Call `pass.load_previous()` on the downstream Pass to keep the
  previous frame contents around (the wgpu `LoadOp::Load` equivalent).
- Or call `pass.set_clear_color([r, g, b, a])` to choose a specific
  clear value (still a clear; just a different colour).

`Scene::add_pass` accepts each pre-pass independently. There's no
automatic chaining of their attachments. If you need two passes to
share a target, route them through the same `set_target(...)` and use
`load_previous` to compose.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Pass, Renderer, Scene, Vertex};

let renderer = Renderer::new();

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let model = Model::new(mesh, Material::pbr());

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
