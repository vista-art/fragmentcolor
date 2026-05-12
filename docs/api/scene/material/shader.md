# Material::shader

Borrow the underlying `Shader` to drop down to direct uniform manipulation —
the escape hatch when the Material's typed setters don't cover what you need
(camera state, custom uniforms, raw texture binds outside the glTF PBR slots).

The returned reference is the same `Shader` the Material is built around, so
changes propagate immediately. Cloning the Material gives you an independent
Shader copy ([Material](https://fragmentcolor.org/api/scene/material) clone
spawns a fresh duplicate); the borrow returned here is *not* independent.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let material = Material::pbr();
material.shader().set(
    "camera.view_proj",
    [
        [1.0_f32, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ],
)?;
material.shader().set("camera.position", [0.0_f32, 0.0, 5.0])?;
# Ok(())
# }
```
