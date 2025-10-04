import { Shader, Mesh } from "fragmentcolor";

const shader = new Shader(`
  @vertex fn vs_main(@location(0) pos: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 1.0);
  }
  @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }

`);

const mesh = new Mesh();
mesh.addVertex([0.0, 0.0, 0.0]);

// Attach mesh to this shader (errors if incompatible)
shader.addMesh(mesh);

// Renderer will draw the mesh when rendering this pass.
// Each Shader represents a RenderPipeline or ComputePipeline
// in the GPU. Adding multiple meshes to it will draw all meshes
// and all its instances in the same Pipeline.
