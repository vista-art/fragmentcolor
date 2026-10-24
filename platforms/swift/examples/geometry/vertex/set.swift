import FragmentColor

let shader = try Shader("""
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>, @location(1) id: u32, @location(2) weight: f32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos * weight, f32(id) * 0.1, 1.0);
  }
  @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }

""")

let vertex = try Vertex([0.0, 0.0]).set("id", 7).set("weight", 0.5)

let mesh = Mesh()
try mesh.addVertex(vertex)

// id uploads as the u32 7 and weight as the f32 0.5.
try shader.addMesh(mesh)