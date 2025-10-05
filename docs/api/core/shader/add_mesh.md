# Shader::add_mesh

Attach a Mesh to this Shader. The Renderer will draw all meshes attached to it (one draw call per mesh, same pipeline).

This method now validates that the mesh's vertex/instance layout is compatible with the shader's @location inputs and returns Result<(), ShaderError>.

- On success, the mesh is attached and will be drawn when this shader is rendered.
- On mismatch (missing attribute or type mismatch), returns an error and does not attach.

## See also

Use Shader::validate_mesh for performing a compatibility check without attaching.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Shader, Mesh};

let shader = Shader::new(r#"
  @vertex fn vs_main(@location(0) pos: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 1.0);
  }
  @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
"#)?;

let mesh = Mesh::new();
mesh.add_vertex([0.0, 0.0, 0.0]);

// Attach mesh to this shader (errors if incompatible)
shader.add_mesh(&mesh)?;

// Renderer will draw the mesh when rendering this pass.
// Each Shader represents a RenderPipeline or ComputePipeline
// in the GPU. Adding multiple meshes to it will draw all meshes
// and all its instances in the same Pipeline.

# Ok(())
# }
```
