import org.fragmentcolor.*

val shader = Shader("""
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

val m1 = Mesh()
m1.addVertex(Vertex(listOf(0.0f, 0.0f)))
val m2 = Mesh()
m2.addVertex(Vertex(listOf(0.5f, 0.0f)))

shader.addMesh(m1)
shader.addMesh(m2)

shader.removeMeshes(listOf(m1, m2))