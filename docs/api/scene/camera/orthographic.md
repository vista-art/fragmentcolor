# Camera::orthographic

Construct a Camera with an orthographic projection. The six arguments are
the frustum planes in view space: `left`, `right`, `bottom`, `top`, `near`,
`far`. Use this when you need a flat 2D look (UI overlays, 2D-in-3D
gameplay, isometric scenes) or for shadow-map style passes where you want
parallel projection.

Built on `glam::Mat4::orthographic_rh`, which targets wgpu's NDC depth
range `[0, 1]` — pair with a depth attachment configured for that range
when you add the Camera to a Pass that does depth testing.

The view component starts at identity: eye at the world origin, looking
down `-Z`, with `+Y` up. Chain [`look_at`](https://fragmentcolor.org/api/scene/camera/look_at)
to position the camera before binding it.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Camera;

// A 16:9 viewport, 10 world units tall, depth range 0.1..100.
let camera = Camera::orthographic(-8.0, 8.0, -4.5, 4.5, 0.1, 100.0);

# let m = camera.view_proj();
# // Orthographic preserves parallel lines: the bottom-right corner is the
# // projection-only [3][3] term, which is 1.0 (unlike perspective's 0).
# assert!((m[3][3] - 1.0).abs() < 1.0e-5);
# Ok(())
# }
```
