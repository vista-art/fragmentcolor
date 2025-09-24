# Shader::add_mesh

Attach a Mesh to this Shader. The Renderer will draw all meshes attached to it (one draw call per mesh, same pipeline).

This method now validates that the mesh’s vertex/instance layout is compatible with the shader’s @location inputs and returns Result<(), ShaderError>.

- On success, the mesh is attached and will be drawn when this shader is rendered.
- On mismatch (missing attribute or type mismatch), returns an error and does not attach.

See also: Shader::validate_mesh for performing a compatibility check without attaching.

## Example

```rust
use fragmentcolor::{Pass, Shader, Mesh, Vertex};

let shader = Shader::default();
let pass = Pass::from_shader("p", &shader);

let mut mesh = Mesh::new();
mesh.add_vertex(Vertex::new([0.0, 0.0]));

// Attach mesh to this shader (errors if incompatible)
shader.add_mesh(&mesh).expect("mesh is compatible");

// Renderer will draw the mesh when rendering this pass.
// Each Shader represents a RenderPipeline or ComputePipeline
// in the GPU. Adding multiple meshes to it will draw all meshes
// and all its instances in the same Pipeline.
```
