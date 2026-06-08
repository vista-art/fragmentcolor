# Scene::add_pass

Append a user-built [Pass](https://fragmentcolor.org/api/core/pass) to the
scene's pass graph. A `Scene` owns one ordered list of passes, and `add_pass`
pushes onto the end. Order is the order you build it: passes render in vec
order, and the pass that absorbs `scene.add(&model)` geometry is an ordinary
member of that list, slotted in at the point of your first `add`.

That makes `add_pass` the hook for shadow maps, depth pre-passes, screen-space
backdrops, and post-effects alike. Insert before your geometry for a backdrop,
after it for an overlay. To inspect or reorder the result, reach for
[`list_passes`](https://fragmentcolor.org/api/scene/scene/list_passes),
[`get_pass`](https://fragmentcolor.org/api/scene/scene/get_pass),
[`remove_pass`](https://fragmentcolor.org/api/scene/scene/remove_pass), and
[`set_passes`](https://fragmentcolor.org/api/scene/scene/set_passes).

The Pass is cloned (shallow Arc-share) when stored, so further changes you
make to the original handle reach the Scene's copy too.

### Each new Pass clears by default

Every `Pass::new(name)` starts with a clear-to-transparent input. The first
render of the frame for that Pass wipes its colour attachment to
`[0, 0, 0, 0]`. When you chain passes that read each other's output, the
downstream Pass clears the upstream Pass's output unless you opt out:

- Call `pass.load_previous()` on the downstream Pass to keep the previous
  contents around (the wgpu `LoadOp::Load` equivalent).
- Or call `pass.set_clear_color([r, g, b, a])` to choose a specific clear
  value (still a clear, just a different colour).

`Scene::add_pass` stores each pass independently. There's no automatic
chaining of their attachments. To share a target between two passes, route
them through the same `set_target(...)` and use `load_previous` to compose.

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
