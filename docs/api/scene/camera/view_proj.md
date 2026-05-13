# Camera::view_proj

Read the combined `proj * view` matrix as a column-major 4x4. Column-major
matches WGSL's `mat4x4<f32>` storage and glam's `to_cols_array_2d()`, so
the result is ready to feed directly into a Shader's `camera.view_proj`
uniform via `Shader::set(...)` if you need direct control. For the common
case, use [`Camera::bind`](https://fragmentcolor.org/api/scene/camera/bind)
to write both `view_proj` and `position` in one call.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Camera;

let camera = Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0)
    .look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

let m = camera.view_proj();
# // Column 3 (translation) reflects the eye offset baked into the view matrix.
# assert!(m[3][2] != 0.0);
let _ = m;
# Ok(())
# }
```
