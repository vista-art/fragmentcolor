# Camera::view_proj

Read the combined `proj * view` matrix as a column-major 4x4. Column-major
matches WGSL's `mat4x4<f32>` storage and glam's `to_cols_array_2d()`, so
the result is ready to feed directly into a Shader's `camera.view_proj`
uniform via `Shader::set(...)` if you need direct control. For the common
case, pass the Camera to
[`Pass::add`](https://fragmentcolor.org/api/core/pass#add). The Pass seeds
`camera.view_proj` + `camera.position` on every Material attached to it
and keeps them in sync with later updates.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Camera;

let camera = Camera::perspective(60.0.to_radians(), 16.0 / 9.0, 0.1, 100.0)
    .look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

let view_proj = camera.view_proj();
# // Column 3 (translation) reflects the eye offset baked into the view matrix.
# assert!(view_proj[3][2] != 0.0);
# let _view_proj = view_proj;
# Ok(())
# }
```
