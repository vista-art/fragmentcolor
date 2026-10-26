# Pass::add

Absorb any `SceneObject` (a
[Model](https://fragmentcolor.org/api/scene/model),
[Camera](https://fragmentcolor.org/api/scene/camera),
[Light](https://fragmentcolor.org/api/scene/light), or, in Rust, a
user-defined node that implements the trait) into the pass. The other
bindings take the built-in kinds. Each kind brings its
own attach behavior. A Model queues a draw with its own per-instance
transform. A Camera writes its uniforms into every shader on the pass and
into the shaders that later Models bring. A Light reaches every shader on
the pass that declares the light roots, both the ones already there and
the ones added afterwards, through a Model or
[`add_shader`](https://fragmentcolor.org/api/core/pass#add_shader).

Camera and Light hold Arc-shared state, so a later `camera.look_at(...)`
or `light.set_color(...)` reaches those shaders with no further `add`
call.

The pass holds every Light added to it, so a Light keeps lighting after
the caller drops its handle.

Returns `Result<&Pass, PassError>`. Models can fail at attach time when
the Mesh layout doesn't match the Material's shader, and Cameras always
succeed. A Light fails with `PassError::LightCapReached { cap }` when a
shader on the pass already holds as many lights as the device allows,
and a Model fails the same way when its shader would take more than that
from the pass. See [Light](https://fragmentcolor.org/api/scene/light)
for the cap on each kind of device. Chain with `?` between calls.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Light, Material, Mesh, Model, Pass, Renderer, Vertex};

let renderer = Renderer::new();

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let model = Model::new(mesh, Material::pbr());

let camera = Camera::perspective(1.047, 1.0, 0.1, 100.0)?
    .look_at([0.0, 0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])?;
let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);

let pass = Pass::new("scene");
pass.add(&model)?;
pass.add(&camera)?;
pass.add(&sun)?;

// Updating the camera later is enough. Every Model already on the pass
// picks up the new view_proj at the next render.
camera.look_at([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])?;
# let _ = renderer;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
