from fragmentcolor import Mesh, Shader, Vertex

shader = Shader("""
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>, @location(1) id: u32, @location(2) weight: f32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos * weight, f32(id) * 0.1, 1.0);
  }
  @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }

""")

vertex = Vertex([0.0, 0.5]).set("id", 7).set("weight", 0.5)

mesh = Mesh()
mesh.add_vertices([
    vertex,
    Vertex([-0.5, -0.5]).set("id", 8).set("weight", 1.0),
    Vertex([0.5, -0.5]).set("id", 9).set("weight", 1.0),
])

# The first id uploads as the u32 7 and its weight as the f32 0.5.
shader.add_mesh(mesh)