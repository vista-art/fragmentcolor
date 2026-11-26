# ToonMaterial::shader

Borrow the underlying `Shader`, for a uniform the material's setters don't
cover. It is the shader the material draws with, so a change reaches every
Model that uses the material, and every clone of it.

For the camera and the lights, add a
[Camera](https://fragmentcolor.org/api/scene/camera) and
[Lights](https://fragmentcolor.org/api/scene/light) to the
[Pass](https://fragmentcolor.org/api/core/pass) that draws the material,
rather than setting `camera.*` by hand. They keep the shader up to date.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::ToonMaterial;

let material = ToonMaterial::new();
material.shader().set("material.alpha_cutoff", 0.25)?;
# Ok(())
# }
```
