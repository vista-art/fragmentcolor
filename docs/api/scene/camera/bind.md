# Camera::bind

Write the camera state into a `Shader`. Sets `camera.view_proj`
(`proj * view`, column-major) and `camera.position` (the world-space eye)
in one call. The typical target is a Material's shader — both `Material::pbr`
and any custom shader that follows the PBR uniform naming will pick the
values up immediately.

The call is best-effort: if the target shader doesn't declare one of the
two uniforms (e.g. a 2D Material that only needs a projection matrix), the
underlying `Shader::set` returns an error which is silently demoted to a
`log::debug!`. This matches Material's own setters — you can call `bind`
defensively without worrying about the shader's surface.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Material, Renderer};

let camera = Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0)
    .look_at([0.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

let renderer = Renderer::new();
let material = Material::pbr(&renderer).await?;
camera.bind(material.shader());

# let m: [[f32; 4]; 4] = material.shader().get("camera.view_proj")?;
# assert!(m != [
# 	[1.0, 0.0, 0.0, 0.0],
# 	[0.0, 1.0, 0.0, 0.0],
# 	[0.0, 0.0, 1.0, 0.0],
# 	[0.0, 0.0, 0.0, 1.0],
# ]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
