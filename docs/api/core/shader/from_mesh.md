# Shader::from_mesh

Build a basic WGSL shader source from the first vertex in a Mesh.

The resulting shader automatically adds the provided Mesh to its internal list of Meshes to render,
so the user doesn't need to call `Shader::add_mesh` manually.

This function uses the **first Vertex** to infer position dimensionality and optional properties.

It generates a minimal vertex shader that consumes `@location(0)` position and a fragment shader that returns a flat color by default. If a `color: vec4<f32>` property exists, it is passed through to the fragment stage and used as output.

## Errors

`from_mesh` returns `Result<Shader, ShaderError>`. A mesh with no vertices returns
`ShaderError::InvalidKey`, since there is no first vertex to build from. A mesh the new
shader refuses returns the error `Shader::add_mesh` gives, such as a vertex whose position
has another dimension than the first one's.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mesh, Shader};

let mut mesh = Mesh::new();
mesh.add_vertex([0.0, 0.0, 0.0]);
let shader = Shader::from_mesh(&mesh)?;

# let _ = shader;
# Ok(())
# }
```
