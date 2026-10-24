import org.fragmentcolor.*

val shader = Shader("""
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>, @location(1) offset: vec2<f32>, @location(2) layer: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos + offset, f32(layer) * 0.1, 1.0);
  }
  @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }

""")

val mesh = Mesh()
mesh.addVertex(listOf(0.0f, 0.0f))

val instance = Instance().set("offset", listOf(0.5f, 0.0f)).set("layer", 2)
mesh.addInstance(instance)

// offset uploads as a vec2<f32> and layer as the u32 2.
shader.addMesh(mesh)