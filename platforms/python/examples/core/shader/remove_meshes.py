from fragmentcolor import Shader, Mesh

shader = Shader("""
  struct VOut { @builtin(position) pos: vec4<f32> };
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>) -> VOut {
    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    return out;
  }
  @fragment
  fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.0,0.0,0.0,1.0); }

""")

m1 = Mesh()
m1.add_vertex([0.0, 0.0])
m2 = Mesh()
m2.add_vertex([0.5, 0.0])

shader.add_mesh(m1)
shader.add_mesh(m2)

shader.remove_meshes([m1, m2])