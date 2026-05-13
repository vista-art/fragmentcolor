# Material::shader

Borrow the underlying `Shader` to drop down to direct uniform manipulation —
the escape hatch when the Material's typed setters don't cover what you need
(camera state, custom uniforms, raw texture binds outside the glTF PBR slots).

The returned reference is the same `Shader` the Material is built around, so
changes propagate immediately to every Model that uses this Material. If you
want a Material variant with different state, build a fresh
`Material::pbr().<setters>` rather than cloning — `Material` clones share
their Shader handle (Arc-clone) so mutations are visible across all clones.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let material = Material::pbr()?;
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
