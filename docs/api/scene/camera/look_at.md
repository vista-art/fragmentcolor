# Camera::look_at

Position the camera in world space. `eye` is where the camera is, `target`
is the point it aims at, and `up` is the world-space up vector that
orients the roll (almost always `[0, 1, 0]`). Returns a handle to the
same Camera (Arc-shared backing) so it chains cleanly off a `perspective`
or `orthographic` constructor and can be called again after the Camera
has been added to a Material — the new view propagates live.

Internally builds the view matrix with `glam::Mat4::look_at_rh`. The
result is a right-handed view matrix that pairs with the right-handed
projection produced by [`perspective`](https://fragmentcolor.org/api/scene/camera/perspective)
and [`orthographic`](https://fragmentcolor.org/api/scene/camera/orthographic).

The eye position is cached on the Camera and exposed via
[`position`](https://fragmentcolor.org/api/scene/camera/position) so
shaders that need it (specular highlights, fresnel) don't have to invert
the view matrix on every frame.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Camera;

let camera = Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0)
    .look_at([0.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

# assert_eq!(camera.position(), [0.0, 1.0, 5.0]);
# Ok(())
# }
```
