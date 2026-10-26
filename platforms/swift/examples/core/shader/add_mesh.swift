import FragmentColor

let shader = try Shader("""
  @vertex fn vs_main(@location(0) pos: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 1.0);
  }
  @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }

""")

let mesh = Mesh()
try mesh.addVertex([0.0, 0.0, 0.0])

// Attach mesh to this shader (errors if incompatible)
try shader.addMesh(mesh)

// Renderer will draw the mesh when rendering this pass.
// Meshes that lay out their vertices alike draw with one
// RenderPipeline, and each other layout gets one of its own.