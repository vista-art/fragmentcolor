# Shader::from_vertex

Build a basic WGSL shader source from a single Vertex layout.

This inspects the vertex position dimensionality (2D or 3D) and optional properties.
It generates a minimal vertex shader that consumes `@location(0)` position and a fragment shader that returns a flat color by default.
If a `color: vec4<f32>` property exists, the shader reads it at `@location(1)`, whatever its place among the vertex's properties, and uses it as the output.

This is intended as a fallback and for quick debugging. Canonical usage is the opposite: write your own shader and then build Meshes that match it.

It returns `Result<Shader, ShaderError>`, like `Shader::new`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Shader, Vertex};

let vertex = Vertex::new([0.0, 0.0, 0.0]);
let shader = Shader::from_vertex(&vertex)?;

# let _ = shader;
# Ok(())
# }
```
