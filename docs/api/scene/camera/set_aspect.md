# Camera::set_aspect

Update the camera's aspect ratio (width / height) in place. The
projection matrix recomputes and propagates to every shader the Camera
was added to, plus the Pass-level camera snapshot the renderer reads for
transparency depth-sorting. No need to drop and recreate the Camera
handle.

Typical use: window-resize handler. On `WindowEvent::Resized`, call
`camera.set_aspect(width as f32 / height as f32)` and the next frame
renders without distortion.

Returns a handle to the same Camera (Arc-shared backing) for chaining.

Behaviour by projection kind:

- **Perspective**: rebuilds from `fovy_radians / near / far` with the
  new aspect. The vertical FOV is preserved, horizontal grows or
  shrinks.
- **Orthographic**: keeps the current vertical extent and rescales the
  horizontal extents so `(right - left) / (top - bottom)` matches the
  new aspect, centred on the existing horizontal midpoint. The frustum
  height stays put; the width tracks the window.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Camera;

let camera = Camera::perspective(60.0.to_radians(), 1.0, 0.1, 100.0);

// Window resize: 1920×1080 → wide-screen aspect.
camera.set_aspect(1920.0 / 1080.0);
# Ok(())
# }
```
