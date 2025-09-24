# Shader::from_mesh

Build a basic WGSL shader source from the first vertex in a Mesh.

This uses the first Vertex to infer position dimensionality and optional properties.
It generates a minimal vertex shader that consumes `@location(0)` position and a fragment shader that returns a flat color by default. If a `color: vec4<f32>` property exists, it is passed through to the fragment stage and used as output.

This is intended as a fallback and for quick debugging. Canonical usage is the opposite: write your own shader and then build Meshes that match it.

## Example

```rust
# pub fn main() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Mesh, Shader, Vertex};

let mut mesh = Mesh::new();
mesh.add_vertex(Vertex::new([0.0, 0.0, 0.0]));
let shader = Shader::from_mesh(&mesh)?;

# let _ = shader;
# Ok(())
# }
```