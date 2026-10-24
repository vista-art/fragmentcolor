import FragmentColor

let shader = try Shader("""
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>, @location(1) offset: vec2<f32>, @location(2) layer: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos + offset, f32(layer) * 0.1, 1.0);
  }
  @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }

""")

let mesh = Mesh()
try mesh.addVertex([0.0, 0.0])

let instance = try Instance().set("offset", [0.5, 0.0]).set("layer", 2)
try mesh.addInstance(instance)

// offset uploads as a vec2<f32> and layer as the u32 2.
try shader.addMesh(mesh)