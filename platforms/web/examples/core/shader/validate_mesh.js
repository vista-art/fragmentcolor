import { Shader, Pass, Mesh } from "fragmentcolor";

const shader = new Shader(`
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@location(0) pos: vec3<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 1.0);
  return out;
}
@fragment fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }

`);
const pass = new Pass("p"); pass.addShader(shader);

const mesh = new Mesh();
mesh.addVertices([
  [-0.5, -0.5, 0.0],
  [ 0.5, -0.5, 0.0],
  [ 0.0,  0.5, 0.0],
]);

shader.validateMesh(mesh); // Ok;
pass.addMesh(mesh);
