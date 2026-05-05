import org.fragmentcolor.*

val shader = Shader("""
struct VOut { @builtin(position) pos: vec4<f32> }
@vertex fn vs_main(@location(0) pos: vec3<f32>) -> VOut {
  var out: VOut
  out.pos = vec4<f32>(pos, 1.0)
  return out
}
@fragment fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }

""")
val pass = Pass("p"); pass.addShader(shader)

val mesh = Mesh()
mesh.addVertices(listOf(Vertex(listOf(-0.5f, -0.5f, 0.0f)), Vertex(listOf(0.5f, -0.5f, 0.0f)), Vertex(listOf(0.0f, 0.5f, 0.0f))))

shader.validateMesh(mesh); // Ok
pass.addMesh(mesh)