# Camera::perspective

Construct a Camera with a perspective projection. `fovy_radians` is the
vertical field of view (use `degrees.to_radians()` if you're starting from
degrees); `aspect` is width / height; `near` and `far` clip the depth range.

Built on `glam::Mat4::perspective_rh`, which targets wgpu's NDC depth
range `[0, 1]`. Pair with a depth attachment configured for that range
when you add the Camera to a Pass that does depth testing.

The view component starts at identity: eye at the world origin, looking
down `-Z`, with `+Y` up. Chain [`look_at`](https://fragmentcolor.org/api/scene/camera/look_at)
to position the camera before binding it.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Camera;

let camera = Camera::perspective(60.0.to_radians(), 16.0 / 9.0, 0.1, 100.0);

# // Perspective collapses the depth axis into a non-trivial matrix; the
# // [2][3] term encodes the -1 wgpu uses for the homogeneous w divide.
# let m = camera.view_proj();
# assert!((m[2][3] + 1.0).abs() < 1.0e-5);
# Ok(())
# }
```
