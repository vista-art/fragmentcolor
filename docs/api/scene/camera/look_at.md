# Camera::look_at

Position the camera in world space. `position` is where the camera sits,
`target` is the point it aims at, and `up` is the world-space up vector
that orients the roll (almost always `[0, 1, 0]`). Matches `Light`'s
`position` vocabulary so both scene types describe "where this object is
in the world" the same way.

Returns a handle to the same Camera (Arc-shared backing) so it chains
cleanly off a `perspective` or `orthographic` constructor and can be
called again after the Camera has been added to a Pass. The new view
propagates live to every Material the Pass renders.

The vectors must be finite, `target` must differ from `position`, and
`up` can't be zero or lie on the line of view, forward or back. Any of
those leaves no way to orient the camera, so the call returns
`Err(CameraError::View)` and the Camera keeps its view.

The view is built in double precision, from a normalized `up`, and stored
as `f32`. So an `up` counts as on that line only when the sine of its angle
to it is below 1e-12, and one just off the line still gets the roll its
numbers describe. A top-down camera 10,000 units up, with its target offset
by 0.001, has a sine of 1e-7 and gets an exact roll.

The result is a right-handed view matrix that pairs with the right-handed
projection produced by [`perspective`](https://fragmentcolor.org/api/scene/camera#perspective)
and [`orthographic`](https://fragmentcolor.org/api/scene/camera#orthographic).

The world-space position is cached on the Camera and exposed via
[`position`](https://fragmentcolor.org/api/scene/camera#position) so
shaders that need it (specular highlights, fresnel) don't have to invert
the view matrix on every frame.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Camera;

let camera = Camera::perspective(1.047, 16.0 / 9.0, 0.1, 100.0)?
    .look_at([0.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])?;

# assert_eq!(camera.position(), [0.0, 1.0, 5.0]);
# Ok(())
# }
```
