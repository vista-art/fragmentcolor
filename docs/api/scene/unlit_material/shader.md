# UnlitMaterial::shader

Borrow the underlying `Shader`, for a uniform the material's setters don't
cover. It is the shader the material draws with, so a change reaches every
Model that uses the material, and every clone of it.

For the camera, add a [Camera](https://fragmentcolor.org/api/scene/camera)
to the [Pass](https://fragmentcolor.org/api/core/pass) that draws the
material, rather than setting `camera.*` by hand. The Camera keeps the
shader up to date as it moves.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::UnlitMaterial;

let material = UnlitMaterial::new();
material.shader().set("material.alpha_cutoff", 0.25)?;
# Ok(())
# }
```
