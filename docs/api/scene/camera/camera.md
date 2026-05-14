# Camera

A `Camera` packages the two things every 3D render needs into one object: a
projection (how the view frustum maps to clip space) and a view (where the
camera sits and what it looks at). Pass it to
[`Pass::add`](https://fragmentcolor.org/api/core/pass#add) to wire its
`camera.view_proj` and `camera.position` into every shader the pass renders;
the Camera holds Arc-shared state, so subsequent
[`look_at`](https://fragmentcolor.org/api/scene/camera/look_at) calls
propagate to every shader the Camera has been wired into.

Internally a Camera carries:

- A `proj` matrix built by [`Camera::perspective`](https://fragmentcolor.org/api/scene/camera/perspective)
  or [`Camera::orthographic`](https://fragmentcolor.org/api/scene/camera/orthographic).
  Both use glam's right-handed builders (`Mat4::perspective_rh`,
  `Mat4::orthographic_rh`), which match wgpu's NDC depth range `[0, 1]`.
- A `view` matrix initialized to identity (eye at origin, looking down `-Z`,
  with `+Y` up). Call [`look_at`](https://fragmentcolor.org/api/scene/camera/look_at)
  to position the camera in world space.
- The world-space `position` of the eye — kept alongside the view matrix so
  shaders that need it (specular highlights, fresnel) don't have to invert
  the view matrix on every frame.

The Camera is the user's domain, not the Material's: a Material is "what the
surface looks like under any light from any viewpoint", a Camera is "which
viewpoint we're using right now".

## Methods

| name           | what it does                                              |
| -------------- | --------------------------------------------------------- |
| `perspective`  | construct a perspective Camera from FOV / aspect / near / far |
| `orthographic` | construct an orthographic Camera from the six frustum planes  |
| `look_at`      | position the eye and aim at a target (chainable, live)    |
| `view_proj`    | read `proj * view` as a column-major 4x4                  |
| `position`     | read the world-space eye position                         |

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Material, Mesh, Model, Pass, Renderer, Vertex};

let renderer = Renderer::new();
let camera = Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0)
    .look_at([0.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.5, 1.0])
        .set(Vertex::COLOR0, [1.0, 1.0, 1.0, 1.0])
        .set(Vertex::UV1, [0.0, 0.0]),
);
let model = Model::new(mesh, Material::pbr()?);

let pass = Pass::new("scene");
pass.add(&model)?;
pass.add(&camera);

// Move later — every shader on the pass picks up the new view at the next render.
camera.look_at([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
