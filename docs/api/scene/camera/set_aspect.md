# Camera::set_aspect

Update the perspective camera's aspect ratio (width / height) in place.
The projection matrix recomputes and propagates to every shader the
Camera was added to, plus the Pass-level camera snapshot the renderer
reads for transparency depth-sorting — no need to drop and recreate the
Camera handle.

Typical use: window-resize handler. On `WindowEvent::Resized`, call
`camera.set_aspect(width as f32 / height as f32)` and the next frame
renders without distortion.

Returns a handle to the same Camera (Arc-shared backing) for chaining.

No-op (with a `log::warn!`) when called on an orthographic camera —
"aspect" isn't well-defined for a free frustum. Use
[`Camera::orthographic`](https://fragmentcolor.org/api/scene/camera/orthographic)
to replace the projection wholesale.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Camera;

let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0);

// Window resize: 1920×1080 → wide-screen aspect.
camera.set_aspect(1920.0 / 1080.0);
# Ok(())
# }
```
