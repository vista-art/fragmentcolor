# Shader::from_mesh

Build a basic WGSL shader source from the first vertex in a Mesh.

The resulting shader automatically adds the provided Mesh to its internal list of Meshes to render,
so the user doesn't need to call `Shader::add_mesh` manually.

This function uses the **first Vertex** to infer position dimensionality and optional properties.

It generates a minimal vertex shader that consumes `@location(0)` position and a fragment shader that returns a flat color by default. If a `color: vec4<f32>` property exists, it is passed through to the fragment stage and used as output.

## Empty Mesh Handling

If the Mesh has no vertices, a default shader is returned and a warning is logged.
Because the default shader does not take any vertex inputs, it is compatible with any Mesh.

## Example

```rust
use fragmentcolor::{Mesh, Shader};

let mut mesh = Mesh::new();
mesh.add_vertex([0.0, 0.0, 0.0]);
let shader = Shader::from_mesh(&mesh);

# let _ = shader;
```
