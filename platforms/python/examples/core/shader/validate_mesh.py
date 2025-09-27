from fragmentcolor import Shader, Pass, Mesh, Vertex

wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> }
@vertex fn vs_main(@location(0) pos: vec3<f32>) -> VOut {
  var out: VOut
  out.pos = vec4<f32>(pos, 1.0)
  return out
}
@fragment fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
"#
shader = Shader(wgsl)
rpass = Pass("p"); rpass.add_shader(shader)

mesh = Mesh()
mesh.add_vertices([
  Vertex([-0.5, -0.5, 0.0]),
  Vertex([ 0.5, -0.5, 0.0]),
  Vertex([ 0.0,  0.5, 0.0]),
])

shader.validate_mesh(mesh); # Ok
rpass.add_mesh(mesh)
