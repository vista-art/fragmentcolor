# Pass::add

Absorb any `SceneObject` (a
[Model](https://fragmentcolor.org/api/scene/model),
[Camera](https://fragmentcolor.org/api/scene/camera),
[Light](https://fragmentcolor.org/api/scene/light), or, in Rust, a
user-defined node that implements the trait) into the pass. The other
bindings take the built-in kinds. Each kind brings its
own attach behaviour: a Model queues a draw with its own per-instance
transform; a Camera or Light wires its uniforms into every shader the
pass renders, both the ones already there and the ones added afterwards.

Camera and Light hold Arc-shared state, so subsequent mutations on the
same value (`camera.look_at(...)`, `light.set_color(...)`) propagate to
every shader on the pass with no further `add` call.

Returns `Result<&Pass, PassError>`. Models can fail at attach time when
the Mesh layout doesn't match the Material's shader, and Cameras always
succeed. A Light fails with `PassError::LightCapReached` when a shader
on the pass already holds 32 lights. Lights added before their shaders
join the pass don't fail: a shader past the cap drops the extra ones
with a logged warning instead. Chain with `?` between calls.

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

let camera = Camera::perspective(1.047, 1.0, 0.1, 100.0)
    .look_at([0.0, 0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);

let pass = Pass::new("scene");
pass.add(&model)?;
pass.add(&camera)?;
pass.add(&sun)?;

// Updating the camera later is enough. Every Model already on the pass
// picks up the new view_proj at the next render.
camera.look_at([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
# let _ = renderer;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
