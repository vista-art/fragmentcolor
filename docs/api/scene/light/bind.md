# Light::bind

Write the light state into a `Shader`. Sets `light.direction` (the
world-space travel direction) and `light.color` (linear RGB) in one call.
The typical target is a Material's shader — `Material::pbr` carries both
uniforms by default and feeds them into the Cook-Torrance + Lambert shading
chain.

The call is best-effort: if the target shader doesn't declare one of the
two uniforms (e.g. an unlit shader, or a custom shader that only consumes
`light.color`), the underlying `Shader::set` returns an error which is
silently demoted to a `log::debug!`. Mirrors `Camera::bind` and `Material`'s
setters.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Light, Material, Renderer};

let renderer = Renderer::new();
let material = Material::pbr(&renderer).await?;
let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
sun.bind(material.shader());

# let dir: [f32; 3] = material.shader().get("light.direction")?;
# assert_eq!(dir, [0.3, -1.0, -0.4]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
