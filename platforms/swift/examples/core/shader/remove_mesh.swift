import FragmentColor

let shader = try Shader("""
  struct VOut { @builtin(position) pos: vec4<f32> }
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>) -> VOut {
    var out: VOut
    out.pos = vec4<f32>(pos, 0.0, 1.0)
    return out
  }
  @fragment
  fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.0,0.0,0.0,1.0); }

""")

let mesh = Mesh()
try mesh.addVertex([0.0, 0.0])
try shader.addMesh(mesh)

// Detach the mesh
try shader.removeMesh(mesh)