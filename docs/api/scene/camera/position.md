# Camera::position

Read the world-space eye position as `[x, y, z]`. This is the value set by
the most recent [`look_at`](https://fragmentcolor.org/api/scene/camera/look_at)
call, or `[0, 0, 0]` if the camera has only been constructed (the default
view is identity, with the eye at the origin).

Shaders that need the eye position (specular highlights, fresnel,
parallax) typically pull it from the `camera.position` uniform written by
[`Camera::bind`](https://fragmentcolor.org/api/scene/camera/bind) — caching
it here keeps every frame cheap (no view-matrix inversion on the GPU side).

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Camera;

let camera = Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0)
    .look_at([3.0, 2.0, 8.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

let eye = camera.position();
# assert_eq!(eye, [3.0, 2.0, 8.0]);
# Ok(())
# }
```
