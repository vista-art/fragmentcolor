import org.fragmentcolor.*

val shader = Shader("""
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>, @location(1) id: u32, @location(2) weight: f32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos * weight, f32(id) * 0.1, 1.0);
  }
  @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }

""")

val vertex = Vertex(listOf(0.0f, 0.0f)).set("id", 7).set("weight", 0.5f)

val mesh = Mesh()
mesh.addVertex(vertex)

// id uploads as the u32 7 and weight as the f32 0.5.
shader.addMesh(mesh)