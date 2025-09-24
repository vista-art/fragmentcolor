# Shader::add_mesh

Attach a Mesh to this Shader. The Renderer will draw all meshes attached to it (one draw call per mesh, same pipeline).

## Example

```rust
use fragmentcolor::{Pass, Shader, Mesh, Vertex};

let shader = Shader::default();
let pass = Pass::from_shader("p", &shader);

let mut mesh = Mesh::new();
mesh.add_vertex(Vertex::new([0.0, 0.0]));

// Attach mesh to this shader
shader.add_mesh(&mesh);

// Renderer will draw the mesh when rendering this pass.
// Each Shader represents a RenderPipeline or ComputePipeline
// in the GPU. Adding multiple meshes to it will draw all meshes
// and all its instances in the same Pipeline.
```