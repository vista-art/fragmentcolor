# Camera::perspective

Construct a Camera with a perspective projection. `fovy_radians` is the
vertical field of view (use `degrees.to_radians()` if you're starting from
degrees); `aspect` is width / height; `near` and `far` clip the depth range.

Built on `glam::Mat4::perspective_rh`, which targets wgpu's NDC depth
range `[0, 1]`. A pass that draws models gets a depth buffer for that
range on its own.

The view component starts at identity: eye at the world origin, looking
down `-Z`, with `+Y` up. Chain [`look_at`](https://fragmentcolor.org/api/scene/camera#look_at)
to position the camera before binding it.

Every value must be finite, `fovy_radians` must lie between 0 and π,
`aspect` can't be 0, and `near` and `far` must be positive and different.
Anything else returns `Err(CameraError::Perspective)`: such values give a
matrix of infinities or NaN, a flipped or collapsed image, or no usable
depth range.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Camera;

let camera = Camera::perspective(1.047, 16.0 / 9.0, 0.1, 100.0)?;

# // Perspective collapses the depth axis into a non-trivial matrix; the
# // [2][3] term encodes the -1 wgpu uses for the homogeneous w divide.
# let m = camera.view_proj();
# assert!((m[2][3] + 1.0).abs() < 1.0e-5);
# Ok(())
# }
```
