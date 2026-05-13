# Pass::add

Absorb a scene-level [Camera](https://fragmentcolor.org/api/scene/camera),
[Light](https://fragmentcolor.org/api/scene/light), or any custom
[`Component`](https://fragmentcolor.org/api/scene) into the pass. The
component is applied immediately to every shader already on the pass
(from prior [`add_model`](https://fragmentcolor.org/api/core/pass/add_model)
calls), and re-applied to every shader added afterwards via `add_model` —
so the ordering of `add` versus `add_model` doesn't matter.

The component is added by reference: later mutations on the same value
(`camera.look_at(...)`, `light.set_color(...)`) propagate to every shader
the pass has wired the component into, with no further `add` call required.

Chainable: returns the Pass so several components can stack on a single
statement.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Light, Material, Mesh, Model, Pass, Renderer, Vertex};

let renderer = Renderer::new();

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.5, 1.0]),
);
let model = Model::new(mesh, Material::pbr(&renderer).await?);

let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
    .look_at([0.0, 0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);

let pass = Pass::new("scene");
pass.add_model(&model)?;
pass.add(&camera).add(&sun);

// Updating the camera later is enough — every Model already on the pass
// picks the new view_proj up at the next render.
camera.look_at([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
