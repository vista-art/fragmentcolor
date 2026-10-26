# Shader::add_mesh

Attach a Mesh to this Shader. The Renderer draws every mesh attached to it, one draw call per mesh. Meshes that lay out their data alike share one pipeline, and a mesh whose stride, attribute offsets or instance stream differ draws with a pipeline of its own.

`add_mesh` checks every vertex and instance of the mesh against the shader's `@location` inputs and returns `Result<(), ShaderError>`.

- An attribute set from numbers takes the type the shader declares at its location: floats for `f32` and `vecN<f32>`, integers for `u32`, `i32` and their vectors when every value is whole and fits.
- On success, the mesh is attached, its attributes keep those types for as long as the mesh lives, and it is drawn when this shader renders.
- On a missing attribute or a value that doesn't fit, the error names the attribute, the declared type and the value, or the vertex or instance that lacks it. The mesh isn't attached and records no type.
- A shader that declares another type for an attribute an earlier shader already typed on this mesh fails to attach.
- A vertex that sets a property named `position` fails with `ShaderError::InvalidKey`, since its position comes from `Vertex::new`.
- A vertex or instance added after the attach must hold every attribute the shader reads. `Renderer.render` returns `MeshError::InvalidVertex` or `MeshError::InvalidInstance` naming the attribute otherwise.
- The `@location` members of a struct argument are inputs too, each read from the attribute of its own name, so `struct VIn { @location(1) color: vec4<f32> }` reads `color`. Built-in members such as `@builtin(vertex_index)` read nothing from the mesh.
- A shader with no `@location` inputs, in its arguments or their members, draws fullscreen and rejects every mesh with `ShaderError::InvalidKey`.

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
// Meshes that lay out their vertices alike draw with one
// RenderPipeline, and each other layout gets one of its own.

# Ok(())
# }
```
